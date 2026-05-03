# OAuth Codex Source Map

Tài liệu này là danh sách source gốc được phép tham khảo khi viết lại Codex bằng Rust trong `oauthcodex`. Không tham khảo code của provider khác, trừ khi file đó được liệt kê ở phần shared dependency vì Codex đang gọi trực tiếp.

## Backend Codex Chính

- `src-tauri/src/models/codex.rs`: model account, token, quota, auth file, JWT payload, quick config.
- `src-tauri/src/models/codex_local_access.rs`: model API Service/local access, routing strategy, stats, cleanup result.
- `src-tauri/src/modules/codex_oauth.rs`: OAuth PKCE, callback server, pending state, manual callback, cancel, token refresh.
- `src-tauri/src/modules/codex_account.rs`: account store, local import, JSON import/export, API key account, profile refresh, switch account, auth injection, quick config, auto-switch, quota alert.
- `src-tauri/src/modules/codex_quota.rs`: ChatGPT usage API call, quota parsing, token refresh-on-failure, batch quota refresh.
- `src-tauri/src/modules/codex_local_access.rs`: local OpenAI-compatible API gateway, account pool routing, model aliasing, chat-completions to responses conversion, SSE conversion, retry/cooldown, stats.
- `src-tauri/src/modules/codex_instance.rs`: instance profile persistence and default settings.
- `src-tauri/src/commands/codex.rs`: public Tauri command surface for account, OAuth, import/export, API key, groups, providers, local access, wakeup.
- `src-tauri/src/commands/codex_instance.rs`: public Tauri command surface for instances, sessions, thread sync, launch commands.

## Backend Codex Phụ Trợ

- `src-tauri/src/modules/codex_session_manager.rs`: session list, token stats, trash/restore.
- `src-tauri/src/modules/codex_session_visibility.rs`: repair Codex session visibility after account/API switch.
- `src-tauri/src/modules/codex_thread_sync.rs`: sync threads across Codex instances.
- `src-tauri/src/modules/codex_wakeup.rs`: Codex CLI detection, task model, execution, cancellation, history.
- `src-tauri/src/modules/codex_wakeup_scheduler.rs`: startup/manual/scheduled wakeup execution.
- `crates/cockpit-core/src/models/codex.rs`: partially extracted Codex model reference.
- `crates/cockpit-core/src/modules/codex_oauth.rs`: older/extracted OAuth implementation; compare with `src-tauri` and treat `src-tauri` as canonical.
- `crates/cockpit-core/src/modules/codex_account.rs`: older/extracted account implementation; compare with `src-tauri` and treat `src-tauri` as canonical.
- `crates/cockpit-core/src/modules/codex_quota.rs`: older/extracted quota implementation.
- `crates/cockpit-core/src/modules/codex_instance.rs`: older/extracted instance implementation.

## Frontend Codex Workflow

- `src/pages/CodexAccountsPage.tsx`: full user workflow for OAuth, API key, token import, local import, switch, local API service, groups, providers, tabs.
- `src/pages/CodexInstancesPage.tsx`: Codex instance page.
- `src/App.tsx`: global Codex navigation, refresh-current event bridge, app-path prompt, launch-on-switch retry, startup local access preparation.
- `src/pages/SettingsPage.tsx`: Codex settings UI: auto refresh, launch paths, local access entry, auto-switch scope, quota alerts.
- `src/pages/WakeupTasksPage.tsx`: legacy/shared wakeup settings references that still include Codex auto refresh and app path values.
- `src/pages/DashboardPage.tsx`: dashboard current Codex account presentation.
- `src/pages/FloatingCardWindow.tsx`: floating-card current Codex account presentation.
- `src/services/codexService.ts`: frontend-to-backend account/OAuth command contract.
- `src/services/codexLocalAccessService.ts`: frontend-to-backend local API service command contract.
- `src/services/codexInstanceService.ts`: frontend-to-backend instance/session command contract.
- `src/services/codexWakeupService.ts`: frontend-to-backend wakeup command contract and camelCase/snake_case mapping.
- `src/services/codexModelProviderService.ts`: model provider persistence contract.
- `src/services/codexAccountGroupService.ts`: Codex account group persistence contract.
- `src/services/dataTransferService.ts`: config/account group/model provider/wakeup/instance import-export contract for Codex.
- `src/services/floatingCardService.ts`: floating-card integration points when Codex account state is shown.
- `src/stores/useCodexAccountStore.ts`: account cache, current account, profile hydration, sync events.
- `src/stores/useCodexInstanceStore.ts`: instance state flow.
- `src/stores/useCodexWakeupStore.ts`: wakeup state flow.
- `src/hooks/useProviderAccountsPage.ts`: shared account-page behavior; use only Codex-applicable patterns and strip provider-generic branches.
- `src/hooks/useAutoRefresh.ts`: full/current Codex quota background refresh scheduler integration.
- `src/hooks/usePlatformRuntimeSupport.ts`: runtime support visibility for Codex-related pages.
- `src/types/codex.ts`: frontend account/quota/session helpers and display derivation.
- `src/types/codexLocalAccess.ts`: frontend local API service types.
- `src/types/codexWakeup.ts`: frontend wakeup types.
- `src/types/platform.ts`: platform id and platform-level contract.

## UI/Presentation Reference

- `src/presentation/platformAccountPresentation.ts`: Codex display labels, quota metrics, plan labels, account presentation.
- `src/components/CodexLocalAccessModal.tsx`: full local API service modal behavior.
- `src/components/CodexLocalAccessModal.css`: local API service styles.
- `src/components/CodexAccountGroupModal.tsx`: Codex group management modal.
- `src/components/CodexGroupAccountPickerModal.tsx`: add Codex accounts to group.
- `src/components/CodexOverviewTabsHeader.tsx`: Codex tabs header.
- `src/components/AccountTagFilterDropdown.tsx`: tag filtering UI used by Codex account list.
- `src/components/AccountFilterDropdown.css`, `src/components/AccountGroupModal.css`, `src/components/GroupAccountPickerModal.css`: reusable Cockpit modal/filter styling patterns; copy only classes used by Codex UI.
- `src/components/codex/CodexQuickConfigCard.tsx`: config.toml quick settings UI.
- `src/components/codex/CodexModelProviderManager.tsx`: API key provider/key manager.
- `src/components/codex/CodexSessionManager.tsx`: session management UI.
- `src/components/codex/CodexWakeupContent.tsx`: wakeup tasks UI.
- `src/components/QuickSettingsPopover.tsx`: quick settings for Codex refresh/config toggles.
- `src/components/SettingsAccountTransferSection.tsx`: config/account transfer entry points involving Codex.
- `src/components/platform/PlatformOverviewTabsHeader.tsx`: Codex overview/provider/wakeup/instance/session tabs.
- `src/components/platform/PlatformInstancesContent.tsx`: Codex instance content wrapper.
- `src/components/platform/PlatformGroupSwitcher.tsx`: group navigation used by Codex account grouping UI.
- `src/components/platform/DosageNotifyQuotaPreview.tsx`, `src/components/platform/DosageNotifyUsageStatus.tsx`: quota/usage display helpers used by platform presentation.
- `src/components/icons/CodexIcon.tsx`, `src/assets/icons/codex.svg`: Codex visual identity.
- `src/styles/pages/codex.css`: page-level Codex CSS.
- `src/styles/pages/settings.css`, `src/styles/pages/settings-extra.css`, `src/styles/settings-shared.css`: settings layout for Codex settings rows.

## Shared Files Codex Gọi Trực Tiếp

- `src-tauri/src/lib.rs`: startup restore for OAuth listener, local access gateway, wakeup scheduler.
- `src-tauri/src/modules/config.rs`: Codex config fields: auto refresh, startup wakeup, app paths, launch-on-switch, local access entry, auto-switch, quota alert.
- `src-tauri/src/modules/web_report.rs`: Codex auto-refresh/report integration and current quota report rows.
- `src-tauri/src/commands/system.rs`: Codex settings command contract: `save_general_config`, `set_app_path`, `set_codex_launch_on_switch`, `set_codex_local_access_entry_visible`, app path detection.
- `src-tauri/src/commands/data_transfer.rs`: Codex config/instance-store import-export bridge.
- `src-tauri/src/modules/macos_native_menu.rs`: native menu Codex refresh/current-account actions.
- `src-tauri/src/modules/oauth_pending_state.rs`: persisted pending OAuth state.
- `src-tauri/src/modules/atomic_write.rs`: safe writes.
- `src-tauri/src/modules/process.rs`: app launch/kill, port cleanup, CLI/process helpers.
- `src-tauri/src/modules/tray.rs`: Codex current account/quota display in tray.
- `src-tauri/src/modules/account.rs`: shared account dispatch and quota alert event.
- `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/instance_store.rs`: generic instance persistence used by Codex.
- `src-tauri/src/modules/logger.rs`: logging semantics.
- `src/utils/codexExportFormats.ts`: export format behavior.
- `src/utils/codexProviderPresets.ts`: API provider presets.
- `src/utils/codexPreferences.ts`: Codex UI preferences.
- `src/utils/codexLocalAccessRiskNotice.tsx`: risk notice behavior.
- `src/utils/accountSyncEvents.ts`: account changed/current changed events.
- `src/utils/autoRefreshScheduler.ts`: scheduler semantics for Codex quota refresh.
- `src/utils/currentAccountRefresh.ts`: per-platform current-account refresh interval map including Codex.
- `src/utils/currentAccountSort.ts`: current Codex account first sorting.
- `src/utils/accountFilters.ts`: tag normalization/filtering used by Codex account list.
- `src/utils/accountsOverviewFilterPersistence.ts`: persisted Codex overview filter fields.
- `src/utils/platformMeta.tsx`: platform metadata and Codex icon/name references.

## Runtime Data Paths To Preserve

- `~/.codex/auth.json`: active Codex auth file.
- `~/.codex/config.toml`: active Codex config.
- `~/.antigravity_cockpit/codex_account_groups.json`: Codex groups.
- `~/.antigravity_cockpit/codex_model_providers.json`: model providers/API keys.
- `~/.antigravity_cockpit/codex_oauth_pending.json`: pending OAuth state.
- `~/.antigravity_cockpit/codex_local_access.json`: local API service collection.
- `~/.antigravity_cockpit/codex_local_access_stats.json`: local API service stats.
- Codex account store files under the current implementation's Cockpit data directory: derive from `src-tauri/src/modules/codex_account.rs`.
- Codex local access collection/stats files: derive exact names from `src-tauri/src/modules/codex_local_access.rs`.
- Codex wakeup tasks/history/runtime config files: derive exact names from `src-tauri/src/modules/codex_wakeup.rs`.
- LocalStorage keys: `agtools.codex.accounts.cache`, `agtools.codex.accounts.current`, `agtools.codex.accounts.overview_layout_mode`, `agtools.codex.accounts.custom_sort_order.v1`, `agtools.codex.local_access_entry_expanded.v1`, `agtools.codex_show_code_review_quota`, `codexApiSwitchVisibilityNoticeDismissed`, and `agtools.current_account_refresh_minutes.v1`.
