# OAuth Codex Implementation Workflow

## Daily Loop

1. Read `oauthcodex/RULES.md`.
2. Pick exactly one task from `oauthcodex/PLAN.md`.
3. Open only the listed source paths from `oauthcodex/SOURCE_MAP.md`.
4. Write or update tests first.
5. Implement the minimum Rust/UI code in `oauthcodex`.
6. Run the task recheck commands.
7. Update the phase checklist in `PLAN.md`.
8. Commit or leave a clean diff summary before moving on.

## Required Recheck Per Task

Run these after every implementation task:

```bash
cargo fmt --manifest-path oauthcodex/Cargo.toml --check
cargo test --manifest-path oauthcodex/Cargo.toml
```

Run this once the crate compiles cleanly:

```bash
cargo clippy --manifest-path oauthcodex/Cargo.toml --all-targets -- -D warnings
```

If a task touches the Codex-only UI, also run:

```bash
npm --prefix oauthcodex/ui run typecheck
npm --prefix oauthcodex/ui run build
npm --prefix oauthcodex/ui run test
```

Create these scripts when the UI package is introduced.

If a task touches async HTTP, local server, process launch, file IO, TOML, SQLite, or scheduler behavior, also run the matching integration test target:

```bash
cargo test --manifest-path oauthcodex/Cargo.toml --test oauth_flow
cargo test --manifest-path oauthcodex/Cargo.toml --test account_store
cargo test --manifest-path oauthcodex/Cargo.toml --test local_access_gateway
cargo test --manifest-path oauthcodex/Cargo.toml --test wakeup_scheduler
cargo test --manifest-path oauthcodex/Cargo.toml --test codex_instances
cargo test --manifest-path oauthcodex/Cargo.toml --test config_contract
cargo test --manifest-path oauthcodex/Cargo.toml --test data_transfer
cargo test --manifest-path oauthcodex/Cargo.toml --test ui_contract
```

Create the test target when the phase introduces that subsystem.

## Multi-Pass Recheck

Every phase must pass three review passes before marking complete:

### Pass 1: Source Parity

- Compare each implemented API with the exact source path listed in `SOURCE_MAP.md`.
- Confirm constants, file names, request URLs, event names, serialized field names, and error branches.
- Update `PLAN.md` parity notes with any intentional difference.

### Pass 2: Failure Modes

Recheck these repeatedly:

- OAuth: port `1455` busy, invalid callback path, missing code, state mismatch, timeout, cancel, stale login id, retry token exchange.
- Token: expired JWT, refresh without new `id_token`, refresh token rotation, refresh failure requiring reauth.
- Account store: corrupt index, missing account file, duplicate account id, disk-full-like write failure, batch import partial failure.
- Group/provider stores: invalid root array, duplicate group/provider base URL, empty group name, stale account ids, provider with no API keys, duplicate API key value, old `preset_` cleanup.
- API key: empty key, URL pasted into key field, invalid Base URL, same key/Base URL, custom provider id/name derivation, default OpenAI URL normalization.
- Quota: API key account no-op, no current account, 401/403 refresh, usage API schema changes, missing windows, code review quota, retry-after quota errors, post-refresh auto-switch/alert once-only guard.
- Local access: wrong bearer key, no accounts, FREE restriction, API key account rejected, port 0 normalization, occupied port, model alias rewrite, large/malformed request, stream/non-stream conversion, upstream retry/cooldown, stats flush.
- Instance/session: running instance protection, stale PID, missing app path, CLI launch mode, copy/empty/existing init, backup creation, session visibility repair, trash/restore, SQLite read-only failures.
- Wakeup: missing CLI/node, configured invalid path, cancellation, schedule next-run calculation, startup delay, quota-reset schedule, history append/trim.
- Settings/data-transfer: sanitize numeric bounds, preserve unrelated config fields, unresolved account refs, disabled imported wakeup tasks, invalid imported provider/group payloads.

### Pass 3: External Contract

- Match command names and payloads from `src/services/codexService.ts`, `src/services/codexLocalAccessService.ts`, `src/services/codexInstanceService.ts`, and `src/services/codexWakeupService.ts`.
- Match frontend event names: `codex-oauth-login-completed`, `codex-oauth-login-timeout`, `codex:file-import-progress`, wakeup progress events, local access update events.
- Match runtime files under `~/.codex` and `~/.antigravity_cockpit`.
- Match LocalStorage preferences listed in `SOURCE_MAP.md`, including Codex overview layout, custom sort order, code review quota visibility, API switch notice dismissal, and current-account refresh map.
- Match startup hooks in `src-tauri/src/lib.rs`: restore local access gateway, restore pending OAuth listener, start wakeup scheduler, trigger startup wakeup tasks.
- Match global app bridge behavior in `src/App.tsx`: Codex route mapping, current-quota refresh command, app-path missing prompt, launch-on-switch retry, local access restart preparation.
- Match Codex auto-refresh/report behavior in `src-tauri/src/modules/web_report.rs`, including interval registration and Codex report rows.
- Match UI scope in `oauthcodex/UI_PLAN.md`: only Codex routes/screens remain, Cockpit visual patterns are preserved, and non-Codex provider UI is removed.

## Phase Gate Template

At the end of each phase, append a note to `PLAN.md`:

```markdown
#### Phase N Gate

- Source parity checked: yes/no
- Tests run:
  - `cargo test --manifest-path oauthcodex/Cargo.toml ...`
- Known gaps:
  - none / list
- Next phase allowed: yes/no
```

## Stop Conditions

Stop and ask before proceeding if:

- Source gốc has conflicting behavior between `src-tauri` and frontend service contracts.
- A feature requires reading non-Codex provider source not listed in `SOURCE_MAP.md`.
- Implementing parity would write outside `oauthcodex` or alter existing app behavior.
- A test would require real OpenAI credentials; replace with fixture/mock and document the manual test instead.
