# gorp

A fork of [Warp](https://github.com/warpdotdev/warp) with the AI, auth,
telemetry, and onboarding surfaces stripped out.

> [!IMPORTANT]
> Not affiliated with or endorsed by Warp. Use at your own risk.

## Status

Active surgical fork. The work is being done in small, reviewable PRs against
`master` — see open and closed PRs in this repo for the running record.

| Branch | Status |
|---|---|
| `chore/disable-upstream-ci` | merged |
| `chore/rebrand-gorp` | merged |
| `chore/remove-launch-at-login` | merged |
| `chore/remove-auth-and-onboarding` | merged |
| `chore/remove-telemetry` | merged (stub-out, not delete) |
| `chore/remove-billing-and-drive` | this PR (split: billing deleted, drive hidden) |
| `chore/remove-ai` | pending (the big one) |
| `chore/remove-orphan-settings` | pending (cleanup pass) |
| `chore/delete-dead-auth-code` | deferred (see [OPEN_ITEMS.md](OPEN_ITEMS.md)) |
| `chore/delete-dead-telemetry-code` | deferred (see [OPEN_ITEMS.md](OPEN_ITEMS.md)) |
| `chore/delete-dead-drive-code` | deferred (see [OPEN_ITEMS.md](OPEN_ITEMS.md)) |

## What's planned to go

- Warp's coding agent + agent management view + agent conversations
- AI assistant panel, AI input completer, AI search, AI context menus
- Voice input
- Sign-in / sign-up / onboarding / hosted-on-anything (HOA) flows
- Warp Drive and the cloud-sync UI it lives inside
- Billing & usage settings pages
- Server-side telemetry calls (stubbed to no-ops, call sites preserved)
- "Launch at login" registration on macOS/Windows
- All upstream GitHub Actions workflows (renamed to `.disabled`)

## What stays

The whole point of forking Warp instead of using a vanilla terminal is the
stuff Warp built that *isn't* AI:

- **File explorer pane** — a real left-side file tree
- **Split panes** — horizontal + vertical with smooth focus + resize
- **Tools panel** — left-side panel with project explorer, global search, etc.
- **Markdown preview** inline for `.md` files
- **Blocks** — every command + its output as a discrete, scrollable, copyable unit
- **First-class themes** with editable presets
- **Workflows** — saved parameterised commands you can recall and run
- **Editor-grade input** — multi-line editing, vim mode, real history
- **Code review pane** for inline diff/PR review (no AI)

## Building

Requires the Rust toolchain pinned in `rust-toolchain.toml`. macOS arm64 is the
primary supported target; Linux + Windows still build but get less testing.

```sh
cargo check -p warp --bin gorp
cargo build  -p warp --bin gorp --release
```

The crate is still named `warp` internally — only the binary, bundle, and
user-visible branding are renamed. This keeps the diff against
`upstream/master` small and lets us merge upstream changes cleanly.

To produce a macOS `.app` bundle:

```sh
RELEASE_CHANNEL=oss ./script/macos/bundle
```

## Tracking upstream

```sh
git fetch upstream
git merge upstream/master
```

The fork keeps the upstream `Channel::Oss` enum variant and the
`script/{macos,linux,windows}/bundle` "oss" channel as the gorp channel —
internally everything still says "oss", only the user-facing strings (binary
name, bundle name, bundle id, URL scheme) are rebranded to gorp.

## Credits

- [warpdotdev/warp](https://github.com/warpdotdev/warp) — the upstream this fork is based on. All real engineering belongs to them.
- [mxcl/vorp](https://github.com/mxcl/vorp) by [Max Howell](https://mxcl.dev) — prior art on the AI/auth/onboarding removal pattern, used here as a map.

## License

Same as upstream — dual-licensed under [AGPL-3.0](LICENSE-AGPL) and [MIT](LICENSE-MIT).
The MIT license covers the `warpui_core` and `warpui` crates; AGPL-3.0 covers everything else.
