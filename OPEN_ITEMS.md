# Open items

Running list of dead-code follow-ups deferred from the surgical-fork
branches. Each entry has a clear trigger condition for revisiting.

## chore/delete-dead-auth-code

**Status:** deferred from `chore/remove-auth-and-onboarding`
**Trigger to revisit:** after `chore/remove-ai` lands and the AI removal has
collapsed the dependency surface.

The `chore/remove-auth-and-onboarding` branch took the (C) path agreed
in planning: short-circuit every UI entry point via
`crate::terminal_only::is_enabled()`, but leave the underlying modules
in tree so we don't spend two days fixing 109 callers of
`crate::auth::*`.

What is still in the binary as dead-but-compiled code:

- `app/src/auth/` (entire module \u2014 `auth_state`, `auth_manager`,
  `auth_view_modal`, `auth_view_body`, `login_slide`, `web_handoff`,
  `paste_auth_token_modal`, `auth_override_warning_modal`,
  `needs_sso_link_view`, `login_failure_notification`,
  `login_error_modal`, `credentials`, `user`, `user_uid`, `anonymous_id`)
- `app/src/workspace/hoa_onboarding/` (welcome banner, tab config step,
  flow orchestration)
- `crates/onboarding/` (entire crate, all slides + callouts)
- The `init` calls in `app/src/lib.rs` for `auth::init` (registers
  keybindings + standard actions; harmless when no auth view is open)

When we revisit:

1. Most of the 109 `crate::auth::*` consumers come from AI / cloud
   modules deleted in `chore/remove-ai` and the upcoming
   `chore/remove-billing-and-drive`. Surface should drop to ~10\u201320
   real callers.
2. Replace remaining callers' uses of `AuthStateProvider::as_ref`
   with hard-coded "anonymous" stubs (`is_logged_in() -> false`,
   `is_anonymous_or_logged_out() -> true`, etc.) or delete the call
   sites entirely if the consuming feature is also gone.
3. Drop `app/src/auth/`, `app/src/workspace/hoa_onboarding/`, and
   `crates/onboarding/` directories. Remove from
   `Cargo.toml` workspace members and `app/Cargo.toml` deps.
4. Drop `mod auth;`, `mod hoa_onboarding;`, and the `auth::init(ctx)`
   line from `app/src/lib.rs`.
5. `app/src/terminal_only.rs` can stay as a marker constant for any
   future surface that still needs a "this is terminal-only mode"
   signal, or be deleted along with the `is_enabled()` call sites if
   the surface is empty.

Estimated diff once collapsed: ~5\u201310k LOC removed.

## chore/delete-dead-telemetry-code

**Status:** deferred from `chore/remove-telemetry`
**Trigger to revisit:** any time after `chore/remove-ai` and
`chore/remove-billing-and-drive` land. This one has no dependency on
other branches but is the kind of cleanup that's easier when there's
less surface area.

The `chore/remove-telemetry` branch took the (Q3=c) stub-out path:
three defensive guards at the network/disk send sites, one guard on
the collector's scheduler, plus a `cfg(not(test))` no-op on the three
`record_*` functions in `crates/warpui_core/src/telemetry/mod.rs`. The
upstream telemetry plumbing stays in tree.

What is still in the binary as dead-but-compiled code:

- `app/src/server/telemetry/` (entire module — ~9k LOC including the
  7k-line `events.rs` `TelemetryEvent` enum, `collector.rs`,
  `rudder_message.rs`, `secret_redaction.rs`, `context_provider.rs`,
  the `mod_tests`, `events_tests`, `secret_redaction_tests` files).
- `crates/warpui_core/src/telemetry/` (the `EventStore` and macros;
  also accessed by non-telemetry code as a recording sink, so this
  is the more sensitive one to remove).
- `crates/warpui_core/src/app_focus_telemetry.rs` (still gets called
  on every app focus/blur transition, just doesn't enqueue).
- Every `send_telemetry_*!` macro call site throughout the codebase
  (∼1k+ call sites; the macro bodies expand to inert code in this
  fork but they still bloat the compiled output).
- `app/src/server/telemetry/macros.rs` (the macros themselves; can be
  replaced with empty-body equivalents when the upstream is removed).

When we revisit:

1. Decide whether to also drop the per-event-type variants from
   `TelemetryEvent` enum. Many of them describe AI / cloud features
   that will already be gone by then; the remainder describe core
   terminal operations and could be repurposed as a local telemetry
   sink if we ever want one.
2. Delete `app/src/server/telemetry/` directory.
3. Delete `crates/warpui_core/src/telemetry/` and
   `crates/warpui_core/src/app_focus_telemetry.rs`.
4. Replace every `send_telemetry_*!(...)` macro invocation with an
   empty stub (or remove the call entirely if surrounding code can be
   simplified along with it).
5. Drop the `mod telemetry;` declarations in `app/src/server/mod.rs`
   and `crates/warpui_core/src/lib.rs`.
6. Remove the `telemetry_config` field from `ChannelConfig` if no
   consumers remain.

Estimated diff once collapsed: ~10–15k LOC removed, plus a measurable
binary-size drop.

## chore/delete-dead-drive-code

**Status:** deferred from `chore/remove-billing-and-drive`
**Trigger to revisit:** after `chore/remove-ai` lands. Same logic as the
dead-auth follow-up — the 48 callers of `crate::drive::*` are mostly
in AI / cloud / cloud-object code that's about to disappear.

The `chore/remove-billing-and-drive` branch took the (Q-split=C)
path agreed in planning:

- **Billing was deleted** — only one consumer (a per-team
  "shared-objects-creation-denied" modal that never fires in
  terminal-only mode); replaced with a stub no-op method that keeps
  upstream callsites compiling.
- **WarpDrive UI was hidden** — left-panel tab dropped from
  `Workspace::compute_left_panel_views`, the top-level **Drive** menu
  is no longer added to the menu bar, and the **View → Toggle Warp
  Drive** item is skipped.
- **`crate::drive::workflows` was moved** to `crate::workflows::runner`
  so the workflow-runner UI (which is a kept feature) doesn't live
  inside a module tree about to be deleted. 6 importers were rerouted.
- **`app/src/drive/` itself stays in tree** as dead-but-compiled code.

What is still in the binary as dead-but-compiled code:

- `app/src/drive/` minus `workflows/` (entire module — ~20k LOC
  including `panel.rs`, `index.rs`, `sharing/`, `import/`, `export.rs`,
  `items/`, `folders/`, `cloud_object_styling.rs`,
  `cloud_object_naming_dialog.rs`, `cloud_action_confirmation_dialog.rs`,
  `empty_trash_confirmation_dialog.rs`, `drive_helpers.rs`, `settings.rs`)
- `DrivePanel` is still constructed in `LeftPanelView::new` (cheap
  empty handle) so the upstream type stays valid as a `ViewHandle`
  field everywhere it's referenced.
- `crate::workflows::runner::ai_assist` will go with `chore/remove-ai`,
  not this branch (it's AI-shaped, not drive-shaped).
- 48 cross-module callers of `crate::drive::*` for cloud-object,
  persistence, telemetry, URI handler, and search integration paths.
  Most of them flow through AI or cloud-objects modules that
  `chore/remove-ai` will also gut, so the surface should collapse
  naturally before we attempt the physical delete.
- The `is_shared_objects_creation_denied_modal_open` bool stays in
  `WorkspaceState` (it's part of upstream's modal-tracking API used
  by `close_all_modals` / `is_any_non_palette_modal_open`); it's just
  never set to `true` in this fork.

When we revisit:

1. Confirm `chore/remove-ai` (and any later cloud-objects branch)
   collapsed the 48 callers down to a manageable number.
2. Drop `app/src/drive/` directory.
3. Drop `mod drive;` and the `pub use index::DriveIndexVariant;` /
   `pub use panel::{DrivePanel, DrivePanelEvent};` re-exports.
4. Replace the `ViewHandle<DrivePanel>` field in `LeftPanelView` with
   a unit-typed placeholder or remove the field entirely.
5. Delete or move the `WarpDriveSettings`, `WarpDriveSource`,
   `OpenWarpDriveObjectSettings`, `OpenWarpDriveObjectArgs`,
   `CloudObjectTypeAndId`, `DriveObjectType` types that the rest of
   the codebase still references.
6. Sweep the URI handler in `app/src/uri/mod.rs` of
   `drive::OpenWarpDriveObjectSettings` and related drive-URL parsing.
7. Sweep `app/src/server/telemetry/events.rs` of all
   `WarpDrive*` telemetry event variants (they'll be inert by then
   per the `chore/remove-telemetry` branch's stubs, but removing
   them shrinks the enum).
8. Remove the `is_shared_objects_creation_denied_modal_open` field
   from `WorkspaceState` once all consumers are gone (and
   `open_shared_objects_creation_denied_modal` along with it).

Estimated diff once collapsed: ~25–30k LOC removed.

## (Add new entries above this line)
