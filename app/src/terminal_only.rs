//! gorp: terminal-only mode flag.
//!
//! gorp is a Warp fork that ships with auth, onboarding, and (later) AI
//! removed. Rather than ripping the corresponding modules out of the binary
//! wholesale (which would touch ~109 callers of `crate::auth` alone and
//! make upstream merges miserable), we keep the upstream code in tree but
//! make it unreachable from the UI: every entry point that would open an
//! auth modal, run onboarding slides, or surface a HOA banner consults
//! [`is_enabled`] and short-circuits when it returns `true`.
//!
//! This module is intentionally tiny and dependency-free so it can be
//! imported anywhere without creating cycles. The constant is `true` for
//! every gorp build; toggling it to `false` (or building with a different
//! constant) is not supported and will not produce a working Warp binary,
//! since the rebrand is permanent.
//!
//! Pattern adapted from mxcl/vorp.
//!
//! # Open follow-up
//!
//! A later branch (`chore/delete-dead-auth-code`, see `OPEN_ITEMS.md`)
//! will physically delete `app/src/auth/`, `crates/onboarding/`, and the
//! HOA onboarding directory now that the AI removal branch has reduced
//! the dependent surface. Until then, the dead code stays in the binary
//! but is never reached.

/// Whether gorp is running in terminal-only mode. Always `true` in
/// shipping builds; see module docs.
pub(crate) const ENABLED: bool = true;

/// Convenience accessor matching the upstream `vorp` pattern.
#[inline]
pub(crate) fn is_enabled() -> bool {
    ENABLED
}
