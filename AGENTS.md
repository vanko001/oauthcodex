# AGENTS.md

## Project identity

`oauthcodex` is a domain-first Rust library implementing the Codex subset of a larger Tauri app (`cockpit-tools`). Source repo is **not on this machine** — all contracts are derived from `PLAN.md`, `RULES.md`, and `SOURCE_MAP.md`.

## Hard rules

- **Codex-only.** Never reference code, types, constants, or workflows from other providers (Cursor, Gemini, Kiro, Qoder, Trae, Windsurf, Workbuddy, Zed, CodeBuddy, GitHub Copilot).
- **Domain vs adapter split.** `src/domain/` contains pure logic with no I/O. `src/adapters/` handles filesystem, HTTP, process, events. Adapters implement traits defined in domain.
- **All public functions return `Result<T, CodexError>`** (defined in `src/error.rs`).
- **Never log raw tokens, API keys, or JWT contents.** Log only account IDs / email (masked), body lengths, or status codes.
- **Atomic writes only.** Use `adapters/fs_store.rs` helpers for all JSON/TOML/auth file writes.
- **Source repo not on disk.** `cockpit-tools` source files listed in `SOURCE_MAP.md` are canonical for contracts but not locally available. When verifying, trust spec docs over guesses.

## Developer commands

All commands use the repo root as working directory:

```bash
cargo fmt --manifest-path oauthcodex/Cargo.toml --check
cargo test --manifest-path oauthcodex/Cargo.toml
cargo clippy --manifest-path oauthcodex/Cargo.toml --all-targets -- -D warnings
```

Run a single integration test target:

```bash
cargo test --manifest-path oauthcodex/Cargo.toml --test oauth_flow
cargo test --manifest-path oauthcodex/Cargo.toml --test account_store
cargo test --manifest-path oauthcodex/Cargo.toml --test quota
cargo test --manifest-path oauthcodex/Cargo.toml --test codex_instances
cargo test --manifest-path oauthcodex/Cargo.toml --test wakeup_scheduler
cargo test --manifest-path oauthcodex/Cargo.toml --test config_contract
cargo test --manifest-path oauthcodex/Cargo.toml --test data_transfer
cargo test --manifest-path oauthcodex/Cargo.toml --test ui_contract
# local_access_gateway — placeholder, 0 tests (HTTP gateway not built)
```

Order: `fmt -> clippy -> test`. Clippy must pass with `-D warnings` before marking anything complete.

## Architecture

```
src/domain/   — account, oauth, quota, local_access, api_key, model_provider, group,
                config, data_transfer, preferences, instance, session, wakeup
src/adapters/ — fs_store (paths + atomic writes), http_client, oauth_callback_server, events
src/bin/      — CLI for manual testing (oauthcodex)
tests/        — 9 integration test targets, fixtures/ with mock data for each subsystem
```

Key runtime paths (all under user home):
- `~/.codex/auth.json`, `~/.codex/config.toml`
- `~/.antigravity_cockpit/codex_*.json` (account_groups, model_providers, oauth_pending, local_access, local_access_stats)

## Non-obvious conventions

- **OAuth PKCE** on port `1455`, with `login_id` + `state` for stale-callback prevention. Timeout: 300s.
- **Token refresh has per-account lock** to prevent parallel refresh race conditions.
- **API key accounts never call usage/quota APIs.** Must be no-op.
- **Local API Service rejects API key accounts** and optionally rejects FREE-plan accounts.
- **Model provider IDs**: `cmp_*` for providers, `cmk_*` for keys. Old `preset_` providers must be cleaned up.
- **Group IDs**: `cgrp_*`.
- **`serde(rename_all = "camelCase")`** on all serialized structs to match original Tauri/TS contracts.
- **Config setters must preserve unrelated fields** — never rewrite entire Settings payload; only touch Codex fields.
- **Preference/localStorage keys** defined in `src/domain/preferences.rs`; all 8 keys from `SOURCE_MAP.md` §121 are implemented.

## Known gaps (as of Phase 14 audit)

- `adapters/local_gateway.rs` — HTTP proxy server not built (SSE, chat/completions ↔ responses conversion, routing strategies). Placeholder test exists with 0 tests.
- `adapters/process.rs` — process launch/kill adapter not created (can't spawn real apps in tests).
- Auto-refresh background scheduler — tokio interval timer not implemented. Clamping/setter logic exists in `config.rs`.
- Token refresh per-account Mutex lock — not yet implemented in AccountStore.
- Tray, native menu, web-report hooks — Tauri-specific and deferred.
- UI (`oauthcodex/ui/**`) — intentionally deferred (Phase 13 not executed).

## Testing conventions

- Mock fixtures in `tests/fixtures/` grouped by subsystem. All contain fake/test data — no real secrets.
- Use `CodexPaths::for_tests(tmp.path())` to get isolated filesystem paths in tests.
- Network-dependent tests use `wiremock` for mock HTTP servers.
- Integration test targets are listed in `Cargo.toml` `[[test]]` sections.

## References

- `RULES.md` — strict rewrite rules and conventions
- `PLAN.md` — 14-phase implementation plan with gate checklists and parity matrix
- `WORKFLOW.md` — daily workflow, multi-pass recheck procedure
- `SOURCE_MAP.md` — canonical source file inventory and shared dependency list
- `UI_PLAN.md` — deferred Codex-only UI blueprint
