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

## (Add new entries above this line)
