//! Server-side diff state management.
//!
//! [`GlobalDiffStateModel`] manages per-(repo, mode) `LocalDiffStateModel` instances
//! and tracks which connections are subscribed to each. It is stored as a plain
//! struct on `ServerModel` (like `ServerBufferTracker` / `PendingFileOps`).

use std::collections::{HashMap, HashSet};

use warp_util::standardized_path::StandardizedPath;
use warpui::{AppContext, ModelHandle};

use crate::code_review::diff_state::{DiffMetadata, DiffMode, DiffState, LocalDiffStateModel};

use super::protocol::RequestId;
use super::server_model::ConnectionId;

// ── Key type ────────────────────────────────────────────────────────

/// Composite key: each (repo, mode) gets its own `LocalDiffStateModel`.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub(super) struct DiffModelKey {
    pub repo_path: StandardizedPath,
    pub mode: DiffMode,
}

// ── Pending response tracker ────────────────────────────────────────

/// Tracks a `GetDiffState` request that arrived while the model was still loading.
/// The response is sent once `NewDiffsComputed` fires.
pub(super) struct PendingDiffStateResponse {
    pub request_id: RequestId,
    pub conn_id: ConnectionId,
}

// ── GlobalDiffStateModel ────────────────────────────────────────────

/// Manages the lifecycle of server-side `LocalDiffStateModel` instances and
/// per-connection subscription tracking.
///
/// A model is created when the first `GetDiffState` arrives for a given key
/// and dropped when the last connection unsubscribes (or disconnects).
pub(super) struct GlobalDiffStateModel {
    /// One model per (repo, mode). Mode is immutable — pinned at construction.
    states: HashMap<DiffModelKey, ModelHandle<LocalDiffStateModel>>,
    /// Per-connection set of subscribed keys.
    conn_to_keys: HashMap<ConnectionId, HashSet<DiffModelKey>>,
    /// Reverse index: per-key set of subscribed connections. Kept in sync with
    /// `conn_to_keys` so that subscriber lookups and push fan-out are O(1).
    key_to_conns: HashMap<DiffModelKey, HashSet<ConnectionId>>,
    /// Pending `GetDiffState` responses waiting for the model to finish loading.
    pending_responses: HashMap<DiffModelKey, Vec<PendingDiffStateResponse>>,
}

impl GlobalDiffStateModel {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            conn_to_keys: HashMap::new(),
            key_to_conns: HashMap::new(),
            pending_responses: HashMap::new(),
        }
    }

    // ── Model CRUD ──────────────────────────────────────────────────

    pub fn get_model(&self, key: &DiffModelKey) -> Option<&ModelHandle<LocalDiffStateModel>> {
        self.states.get(key)
    }

    pub fn insert_model(&mut self, key: DiffModelKey, model: ModelHandle<LocalDiffStateModel>) {
        self.states.insert(key, model);
    }

    /// Finds any model for the given repo path, regardless of mode.
    /// Used by `DiscardFiles` which operates on the working directory.
    pub fn find_model_for_repo(
        &self,
        repo_path: &StandardizedPath,
    ) -> Option<&ModelHandle<LocalDiffStateModel>> {
        self.states
            .iter()
            .find(|(key, _)| &key.repo_path == repo_path)
            .map(|(_, model)| model)
    }

    pub fn remove_model(&mut self, key: &DiffModelKey) {
        self.states.remove(key);
        self.key_to_conns.remove(key);
        self.pending_responses.remove(key);

        // Clean stale references from the per-connection index.
        for keys in self.conn_to_keys.values_mut() {
            keys.remove(key);
        }
    }

    /// Reads the current `DiffState` and cloned `DiffMetadata` from the model
    /// for `key`. Returns `fallback` values when the model is absent.
    pub fn read_state_and_metadata(
        &self,
        key: &DiffModelKey,
        fallback_state: DiffState,
        app: &AppContext,
    ) -> (DiffState, Option<DiffMetadata>) {
        self.states
            .get(key)
            .map(|model| {
                let m = model.as_ref(app);
                (m.get(), m.metadata().cloned())
            })
            .unwrap_or((fallback_state, None))
    }

    // ── Connection subscription tracking ────────────────────────────

    /// Records that `conn_id` is subscribed to `key`.
    pub fn subscribe_connection(&mut self, key: DiffModelKey, conn_id: ConnectionId) {
        self.conn_to_keys
            .entry(conn_id)
            .or_default()
            .insert(key.clone());
        self.key_to_conns.entry(key).or_default().insert(conn_id);
    }

    /// Removes `conn_id`'s subscription for `key`.
    /// If the key has zero remaining subscribers the model is dropped inline.
    pub fn unsubscribe_connection(&mut self, key: &DiffModelKey, conn_id: ConnectionId) {
        if let Some(keys) = self.conn_to_keys.get_mut(&conn_id) {
            keys.remove(key);
        }

        // Remove any pending responses for this connection on this key,
        // so we don't try to send a GetDiffStateResponse after unsubscribe.
        if let Some(pending) = self.pending_responses.get_mut(key) {
            pending.retain(|p| p.conn_id != conn_id);
        }

        if let Some(conns) = self.key_to_conns.get_mut(key) {
            conns.remove(&conn_id);
            if conns.is_empty() {
                self.remove_model(key);
            }
        }
    }

    /// Removes all subscriptions for a disconnected connection.
    /// Orphaned models (no remaining subscribers) are dropped inline.
    pub fn remove_connection(&mut self, conn_id: ConnectionId) {
        let keys = self.conn_to_keys.remove(&conn_id).unwrap_or_default();

        // Also remove any pending responses for this connection.
        for pending_list in self.pending_responses.values_mut() {
            pending_list.retain(|p| p.conn_id != conn_id);
        }

        for key in keys {
            if let Some(conns) = self.key_to_conns.get_mut(&key) {
                conns.remove(&conn_id);
                if conns.is_empty() {
                    self.remove_model(&key);
                }
            }
        }
    }

    /// Returns the connection IDs subscribed to `key`.
    pub fn subscribed_connections(&self, key: &DiffModelKey) -> Vec<ConnectionId> {
        self.key_to_conns
            .get(key)
            .map(|conns| conns.iter().copied().collect())
            .unwrap_or_default()
    }

    // ── Pending response tracking ───────────────────────────────────

    /// Returns `true` if there are pending responses queued for `key`.
    pub fn has_pending_responses(&self, key: &DiffModelKey) -> bool {
        self.pending_responses
            .get(key)
            .is_some_and(|v| !v.is_empty())
    }

    /// Registers a pending `GetDiffState` response to be sent once the model loads.
    pub fn add_pending_response(
        &mut self,
        key: DiffModelKey,
        request_id: RequestId,
        conn_id: ConnectionId,
    ) {
        self.pending_responses
            .entry(key)
            .or_default()
            .push(PendingDiffStateResponse {
                request_id,
                conn_id,
            });
    }

    /// Drains all pending responses for `key`.
    pub fn drain_pending_responses(&mut self, key: &DiffModelKey) -> Vec<PendingDiffStateResponse> {
        self.pending_responses.remove(key).unwrap_or_default()
    }
}
