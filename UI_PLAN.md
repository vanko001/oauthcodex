# OAuth Codex UI Implementation Plan

**Goal:** Tạo UI Codex-only dựa trên workflow Cockpit hiện tại, giữ đầy đủ tính năng Codex nhưng loại bỏ navigation, settings, state và component không liên quan provider khác.

**Architecture:** UI gọi facade/CLI/Tauri bridge của `oauthcodex` qua một service layer duy nhất. Component chỉ nhận data đã chuẩn hóa từ store; mọi persistence nhạy cảm nằm ở Rust, UI chỉ giữ preference không bí mật trong localStorage.

---

## Source Boundary

Chỉ tham khảo UI/source được liệt kê trong `oauthcodex/SOURCE_MAP.md`. Được tái sử dụng layout, CSS pattern, modal/table/tab behavior từ Cockpit, nhưng phải xóa mọi nhánh provider khác trước khi đưa vào UI mới.

## Target UI Layout

- Create: `oauthcodex/ui/package.json`
- Create: `oauthcodex/ui/src/main.tsx`
- Create: `oauthcodex/ui/src/App.tsx`
- Create: `oauthcodex/ui/src/services/codexClient.ts`
- Create: `oauthcodex/ui/src/stores/useCodexUiStore.ts`
- Create: `oauthcodex/ui/src/pages/CodexAccountsPage.tsx`
- Create: `oauthcodex/ui/src/pages/CodexInstancesPage.tsx`
- Create: `oauthcodex/ui/src/pages/CodexWakeupPage.tsx`
- Create: `oauthcodex/ui/src/pages/CodexSettingsPage.tsx`
- Create: `oauthcodex/ui/src/components/**`
- Create: `oauthcodex/ui/src/styles/codex.css`
- Create: `oauthcodex/ui/tests/ui-contract.spec.ts`

## Keep From Cockpit

- Account overview: list/compact/table modes, current-first sorting, custom sort, pagination, filter by plan/error/tag/group, group-by-tag.
- Add account modal: OAuth start/cancel/manual callback, local auth import, JSON import, API key/provider add.
- Batch actions: refresh quota, export, delete, tag edit, add/move/remove group.
- Local API Service modal: select accounts, port/API key, routing strategy, stats, rotate key, start/stop/restart.
- Provider manager: presets, custom provider, key list, duplicate validation, quick switch.
- Quick config: `config.toml` context window and auto compact token limit.
- Instances/sessions: default and named instances, start/stop/open, launch command, session list, token stats, trash/restore, visibility repair.
- Wakeup: runtime status, task list/editor, schedule presets, run/cancel, history.
- Settings: Codex auto refresh, current-account refresh, app path, launch-on-switch, local access entry visibility, auto-switch scope/thresholds, quota alert thresholds, Codex-only import/export.
- Presentation: Codex icon, plan badges, quota bars, reset labels, quota error state, loading/empty/error states, dark theme parity.

## Remove From Cockpit

- Provider navigation and pages for Cursor, Gemini, Kiro, Qoder, Trae, Windsurf, Zed, CodeBuddy, GitHub Copilot, Workbuddy.
- Shared dashboard cards that compare all providers.
- Settings fields unrelated to Codex, updater/release/signing controls, non-Codex app paths.
- Generic group/model management not used by Codex account groups or Codex provider presets.
- Easter egg, marketing copy, release tooling UI, and any provider-specific warnings not triggered by Codex source.

## Phase UI-1: App Shell And Routing

**Files:**

- Read: `src/App.tsx`
- Read: `src/utils/platformMeta.tsx`
- Create: `oauthcodex/ui/src/App.tsx`

**Tasks:**

1. Implement a Codex-only route set: accounts, instances, wakeup, settings.
2. Keep global refresh-current wiring and app-path missing prompt from `src/App.tsx`.
3. Keep local access restart preparation on startup.
4. Remove all provider route maps except `codex`.
5. Test route switching, startup loading, and missing app path prompt.

## Phase UI-2: Service And Store Contract

**Files:**

- Read: `src/services/codexService.ts`
- Read: `src/services/codexLocalAccessService.ts`
- Read: `src/services/codexInstanceService.ts`
- Read: `src/services/codexWakeupService.ts`
- Read: `src/stores/useCodexAccountStore.ts`
- Create: `oauthcodex/ui/src/services/codexClient.ts`
- Create: `oauthcodex/ui/src/stores/useCodexUiStore.ts`

**Tasks:**

1. Wrap every Codex command behind `codexClient`.
2. Normalize snake_case/camelCase at the service boundary only.
3. Store accounts, current account, groups, providers, instances, sessions, wakeup tasks, local access state, and config.
4. Preserve sync events: account changed, current changed, OAuth completed/timeout, import progress, wakeup progress, local access updates.
5. Test service payloads against `oauthcodex/tests/ui_contract.rs` fixtures.

## Phase UI-3: Accounts Page

**Files:**

- Read: `src/pages/CodexAccountsPage.tsx`
- Read: `src/components/CodexOverviewTabsHeader.tsx`
- Read: `src/components/AccountTagFilterDropdown.tsx`
- Read: `src/components/CodexAccountGroupModal.tsx`
- Read: `src/components/CodexGroupAccountPickerModal.tsx`
- Read: `src/styles/pages/codex.css`
- Create: `oauthcodex/ui/src/pages/CodexAccountsPage.tsx`

**Tasks:**

1. Copy the account overview workflow with only Codex tabs.
2. Implement add OAuth/API-key/import modal states and validation messages.
3. Implement current account switch, rename, tags, quota refresh, export, delete, batch selection.
4. Implement group folders, tag grouping, filters, sort, pagination, and custom sort persistence.
5. Implement empty/loading/error states for no accounts, quota failure, failed import, and OAuth timeout.
6. Verify mobile/table layout does not overflow long email/provider/base URL values.

## Phase UI-4: Local Access And Providers

**Files:**

- Read: `src/components/CodexLocalAccessModal.tsx`
- Read: `src/components/codex/CodexModelProviderManager.tsx`
- Read: `src/utils/codexLocalAccessRiskNotice.tsx`
- Read: `src/utils/codexProviderPresets.ts`
- Create: `oauthcodex/ui/src/components/CodexLocalAccessModal.tsx`
- Create: `oauthcodex/ui/src/components/CodexModelProviderManager.tsx`

**Tasks:**

1. Keep the local access risk notice and API key visibility behavior.
2. Implement account eligibility display: OAuth only, optional FREE restriction, stale account cleanup.
3. Implement port edit, API key rotate/copy, routing strategy, stats, start/stop/restart.
4. Implement provider preset/custom flows, duplicate base URL/key validation, key remove, quick switch.
5. Test wrong bearer key display, occupied port, no eligible accounts, duplicate provider, and hidden entry state.

## Phase UI-5: Instances, Sessions, Wakeup

**Files:**

- Read: `src/pages/CodexInstancesPage.tsx`
- Read: `src/components/platform/PlatformInstancesContent.tsx`
- Read: `src/components/codex/CodexSessionManager.tsx`
- Read: `src/components/codex/CodexWakeupContent.tsx`
- Create: `oauthcodex/ui/src/pages/CodexInstancesPage.tsx`
- Create: `oauthcodex/ui/src/pages/CodexWakeupPage.tsx`

**Tasks:**

1. Keep default/named instance forms and launch/start/stop/open controls.
2. Keep session list, token stats, trash/restore, visibility repair and backup confirmation.
3. Keep wakeup runtime status, task editor, schedule controls, run/cancel, progress, history.
4. Remove cross-provider instance selectors and unsupported runtime sections.
5. Test stale PID, missing app path, missing CLI/node, cancelled wakeup, and corrupt history display.

## Phase UI-6: Settings, Transfer, Presentation

**Files:**

- Read: `src/pages/SettingsPage.tsx`
- Read: `src/components/QuickSettingsPopover.tsx`
- Read: `src/components/SettingsAccountTransferSection.tsx`
- Read: `src/presentation/platformAccountPresentation.ts`
- Read: `src/pages/DashboardPage.tsx`
- Read: `src/pages/FloatingCardWindow.tsx`
- Create: `oauthcodex/ui/src/pages/CodexSettingsPage.tsx`
- Create: `oauthcodex/ui/src/components/CodexQuickSettings.tsx`
- Create: `oauthcodex/ui/src/presentation/codexPresentation.ts`

**Tasks:**

1. Keep Codex settings only: refresh intervals, app path, launch-on-switch, local access entry, auto-switch, quota alerts.
2. Keep Codex-only import/export bundle flow and unresolved-ref reporting.
3. Keep current-account presentation data for dashboard/floating/tray equivalents only if the new UI exposes those surfaces.
4. Remove provider-wide account transfer options and non-Codex settings rows.
5. Test invalid numeric bounds, disabled `-1` refresh, unresolved import refs, and preference persistence.

## UI Verification

Run after UI implementation:

```bash
npm --prefix oauthcodex/ui run typecheck
npm --prefix oauthcodex/ui run build
npm --prefix oauthcodex/ui run test
```

Manual checks:

1. OAuth start/cancel/manual callback modal.
2. Add API key with default and custom provider.
3. Filter/sort/group accounts with long email and many tags.
4. Start/stop local API service and rotate key.
5. Switch account and confirm current marker, tray/dashboard presentation, and app-path prompt.
6. Run wakeup task with mocked progress.
7. Export/import Codex-only config bundle.
8. Check desktop and mobile widths for no clipped buttons or overlapping text.
