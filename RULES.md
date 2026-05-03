# OAuth Codex Rewrite Rules

## Scope

Chỉ rewrite tính năng Codex. Không copy hoặc tham khảo logic provider khác như Cursor, Gemini, Kiro, Qoder, Trae, Windsurf, Workbuddy, Zed, CodeBuddy, GitHub Copilot. Nếu cần shared behavior, chỉ dùng file shared đã liệt kê trong `oauthcodex/SOURCE_MAP.md`.

## Allowed References

1. Source gốc Codex và shared dependency trong `oauthcodex/SOURCE_MAP.md`.
2. Code mới nằm trong `oauthcodex`.
3. Rust standard library và crate public cần thiết cho HTTP, OAuth, JSON, TOML, async, crypto, file locking, SQLite, tests.

Không tham khảo thêm file ngoài danh sách nếu chưa cập nhật `SOURCE_MAP.md` với lý do rõ ràng.

## Canonical Source Priority

1. `src-tauri/src/**/codex*.rs` là nguồn đúng nhất cho backend.
2. `src/pages/CodexAccountsPage.tsx` và `src/services/codex*.ts` là nguồn đúng nhất cho user workflow/API contract.
3. `crates/cockpit-core/src/**/codex*.rs` chỉ dùng để so sánh, không override `src-tauri` nếu khác nhau.

Ví dụ: OAuth scope trong `src-tauri/src/modules/codex_oauth.rs` gồm `openid profile email offline_access api.connectors.read api.connectors.invoke`; nếu `crates/cockpit-core` thiếu connector scopes thì phải theo `src-tauri`.

## Rust Architecture Rules

- Tách domain khỏi adapter: OAuth/account/quota/local access không phụ thuộc Tauri UI.
- Không hardcode home path ngoài một module path resolver.
- File write phải atomic cho JSON/TOML/auth files.
- Token và API key không log raw value; chỉ log account id/email masked hoặc body length/status.
- Mỗi public action phải trả `Result<T, CodexError>` thay vì `String` rời rạc; adapter có thể convert sang string.
- Mọi struct serialized phải giữ tên field tương thích source gốc (`serde(rename_all = "camelCase")` hoặc snake_case đúng contract).
- OAuth state phải chống stale callback bằng `login_id`, `state`, timeout, persisted pending file.
- Token refresh phải có per-account lock để tránh refresh song song ghi đè token.
- API key accounts không được gọi quota OAuth usage API.
- Local API Service chỉ nhận OAuth accounts, loại API key accounts, và có tùy chọn chặn FREE accounts như source gốc.
- Tính năng OpenCode/OpenClaw trong `commands/codex.rs` chỉ được implement nếu vẫn nằm trong Codex switch side-effect; đặt sau trait adapter, không kéo source provider khác.
- Settings/config chỉ implement các field Codex và field shared cần giữ nguyên khi save; không rewrite toàn bộ Settings của provider khác.
- Data-transfer chỉ implement phần Codex: accounts, Codex account groups, model providers, Codex wakeup, Codex instance stores, and Codex auto-switch selected account refs.
- UI preferences/localStorage chỉ copy các key Codex được liệt kê trong `SOURCE_MAP.md`.
- UI mới phải là Codex-only. Có thể tái sử dụng layout/component/CSS pattern của Cockpit từ `SOURCE_MAP.md`, nhưng phải xóa route, store, setting, label, modal, và action của provider khác.
- UI không được giữ màn hình so sánh nhiều provider, navigation provider khác, release/updater settings, hoặc warning không xuất phát từ Codex source.
- UI service layer là nơi duy nhất map command payload; component không gọi trực tiếp backend adapter nếu đã có `codexClient`.

## Feature Parity Rules

Mỗi feature chỉ được đánh dấu done khi có:

1. Mapping source path trong `SOURCE_MAP.md`.
2. Rust module/file hoặc UI file mới trong `oauthcodex`.
3. Unit/integration test hoặc fixture parity test.
4. Recheck thủ công theo `WORKFLOW.md`.
5. Ghi chú parity: implemented, intentionally deferred, hoặc blocked.

Không bỏ qua các edge case đã có trong source gốc: OAuth port busy, manual callback, timeout event, token exchange retry, invalid state, expired pending state, stale login id, expired JWT, malformed JWT, refresh token missing, refresh token rotation, disk full, invalid Base URL, API key nhầm URL, duplicate account/provider/group ids, stale local access account ids, occupied local access port, wrong local bearer key, gateway retry/cooldown, partial import failure, corrupt runtime files, session repair backup, missing CLI/node, cancelled wakeup scope.

## Low-Level Case Rules

- Every file loader must define behavior for missing file, empty file, invalid JSON/TOML, wrong root type, stale IDs, duplicate IDs, and partial write failure.
- Every network call must define behavior for timeout, DNS/connect error, non-2xx, empty body, invalid JSON, missing required fields, and schema drift.
- Every local server must define behavior for bind failure, port conflict, malformed request line, missing content length, large body, unsupported method, CORS preflight, auth failure, upstream disconnect, and graceful shutdown.
- Every process/CLI action must define behavior for missing executable, missing permissions, already running/stopped process, stale PID, unsupported OS, and terminal launch failure.

## Commit/Phase Rules

- Làm theo phase trong `PLAN.md`.
- Mỗi task nhỏ kết thúc bằng test chạy được hoặc ghi rõ lý do chưa chạy.
- Không trộn phase local access với phase OAuth/account nếu chưa qua phase gate.
- Không refactor ngoài `oauthcodex` khi đang rewrite, trừ khi người dùng yêu cầu tích hợp ngược vào app hiện tại.
