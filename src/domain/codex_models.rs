use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CodexAuthMode {
    #[serde(rename = "oauth")]
    OAuth,
    #[serde(rename = "apikey")]
    ApiKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CodexApiProviderMode {
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "custom")]
    Custom,
    #[serde(rename = "azure")]
    Azure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct CodexQuickConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_context_window: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_auto_compact_token_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CodexTokens {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

impl CodexTokens {
    pub fn empty() -> Self {
        Self {
            access_token: None,
            refresh_token: None,
            id_token: None,
            token_type: None,
            expires_at: None,
            scope: None,
        }
    }

    pub fn has_refresh_token(&self) -> bool {
        self.refresh_token.as_ref().is_some_and(|t| !t.is_empty())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexQuotaWindow {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub window_type: Option<String>,
    pub limit: u64,
    pub used: u64,
    pub percentage: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub presence: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexQuotaErrorInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexQuota {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_type: Option<String>,
    #[serde(default)]
    pub windows: Vec<CodexQuotaWindow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_review_quota: Option<CodexQuotaWindow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CodexQuotaErrorInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthFile {
    pub auth_mode: CodexAuthMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<CodexTokens>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtAuthData {
    pub account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    pub plan_type: String,
    #[serde(default)]
    pub organizations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(
        rename = "https://api.openai.com/auth",
        skip_serializing_if = "Option::is_none"
    )]
    pub auth: Option<JwtAuthData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAccount {
    pub id: String,
    pub provider: String,
    pub auth_mode: CodexAuthMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(default)]
    pub organizations: Vec<String>,
    pub display_name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub tokens: CodexTokens,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_provider_mode: Option<CodexApiProviderMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota: Option<Box<CodexQuota>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_refresh: Option<String>,
}

impl CodexAccount {
    pub fn is_oauth(&self) -> bool {
        matches!(self.auth_mode, CodexAuthMode::OAuth)
    }

    pub fn is_api_key(&self) -> bool {
        matches!(self.auth_mode, CodexAuthMode::ApiKey)
    }

    pub fn has_tokens(&self) -> bool {
        self.tokens.access_token.is_some()
    }

    pub fn token_needs_refresh(&self) -> bool {
        self.is_oauth() && self.tokens.refresh_token.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAccountIndex {
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_account_id: Option<String>,
    pub accounts: Vec<CodexAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAccountSummary {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_type: Option<String>,
    pub display_name: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAccountSummaryList {
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_account_id: Option<String>,
    pub accounts: Vec<CodexAccountSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAccountGroup {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub account_ids: Vec<String>,
    pub sort_order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAccountGroupList {
    #[serde(default)]
    pub groups: Vec<CodexAccountGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexModelApiKey {
    pub id: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexModelProvider {
    pub id: String,
    pub name: String,
    pub base_url: String,
    #[serde(default)]
    pub api_keys: Vec<CodexModelApiKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexModelProviderList {
    #[serde(default)]
    pub providers: Vec<CodexModelProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexPendingOAuthState {
    pub login_id: String,
    pub state: String,
    pub code_verifier: String,
    pub code_challenge: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStrategy {
    Auto,
    #[serde(rename = "quota_high_first")]
    QuotaHighFirst,
    #[serde(rename = "quota_low_first")]
    QuotaLowFirst,
    #[serde(rename = "plan_high_first")]
    PlanHighFirst,
    #[serde(rename = "plan_low_first")]
    PlanLowFirst,
    #[serde(rename = "expiry_soon_first")]
    ExpirySoonFirst,
    #[serde(rename = "round_robin")]
    RoundRobin,
    #[serde(rename = "response_affinity")]
    ResponseAffinity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAccessCollection {
    pub version: u32,
    pub accounts: Vec<String>,
    pub port: u16,
    pub enabled: bool,
    pub local_api_key: String,
    pub restrict_free_accounts: bool,
    pub routing_strategy: RoutingStrategy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAccessStatsWindow {
    pub requests: u64,
    pub successes: u64,
    pub failures: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub latency_ms_sum: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAccessStatsSnapshot {
    pub daily: LocalAccessStatsWindow,
    pub weekly: LocalAccessStatsWindow,
    pub monthly: LocalAccessStatsWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAccessStateSnapshot {
    pub enabled: bool,
    pub running: bool,
    pub port: u16,
    pub base_url: String,
    pub account_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub local_api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<LocalAccessStatsSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAccessStatsEvent {
    pub timestamp: String,
    pub account_id: String,
    pub model: String,
    pub status: u16,
    pub latency_ms: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub is_stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAccessStatsFile {
    pub requests: Vec<LocalAccessStatsEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceLaunchMode {
    Auto,
    Manual,
    #[serde(rename = "cli")]
    Cli,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexInstance {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub auth_mode: Option<CodexAuthMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bound_account_id: Option<String>,
    pub follow_local_account: bool,
    pub launch_mode: InstanceLaunchMode,
    #[serde(default)]
    pub extra_args: Vec<String>,
    #[serde(default)]
    pub extra_env: HashMap<String, String>,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexInstanceList {
    pub instances: Vec<CodexInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceLaunchCommand {
    pub id: String,
    pub launch_command: String,
    pub launch_command_shell: String,
    pub launch_command_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexSession {
    pub id: String,
    pub instance_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub token_count: u64,
    pub is_trashed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trash_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub message_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexSessionList {
    pub sessions: Vec<CodexSession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStats {
    pub total_tokens: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub average_per_message: u64,
    pub peak_message_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTokenStats {
    pub id: String,
    pub token_stats: TokenStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityIssue {
    pub session_id: String,
    pub issue: String,
    pub found_in_backup: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityRepairResult {
    pub restored_count: u64,
    pub backup_created: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_path: Option<String>,
    #[serde(default)]
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityRepairReport {
    pub visibility_issues: Vec<VisibilityIssue>,
    pub repair_result: VisibilityRepairResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WakeupScheduleKind {
    Daily,
    Weekly,
    Interval,
    #[serde(rename = "quota_reset")]
    QuotaReset,
    #[serde(rename = "startup_delay")]
    StartupDelay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeupSchedule {
    pub kind: WakeupScheduleKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval_minutes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days_of_week: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeupTask {
    pub id: String,
    pub name: String,
    pub schedule: WakeupSchedule,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeupRuntimeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeupState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_config: Option<WakeupRuntimeConfig>,
    #[serde(default)]
    pub tasks: Vec<WakeupTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeupHistoryEntry {
    pub task_id: String,
    pub run_at: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeupProgressEvent {
    pub task_id: String,
    pub phase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfigCodex {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_auto_refresh_minutes: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_startup_wakeup_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_startup_wakeup_delay_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_app_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_specified_app_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_launch_on_switch: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_restart_specified_app_on_switch: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_local_access_entry_visible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_auto_switch_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_auto_switch_primary_threshold: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_auto_switch_secondary_threshold: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_quota_alert_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_quota_alert_primary_threshold: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_quota_alert_secondary_threshold: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_quota_alert_cooldown_minutes: Option<u64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for UserConfigCodex {
    fn default() -> Self {
        Self {
            codex_auto_refresh_minutes: Some(30),
            codex_startup_wakeup_enabled: None,
            codex_startup_wakeup_delay_seconds: None,
            codex_app_path: None,
            codex_specified_app_path: None,
            codex_launch_on_switch: None,
            codex_restart_specified_app_on_switch: None,
            codex_local_access_entry_visible: None,
            codex_auto_switch_enabled: None,
            codex_auto_switch_primary_threshold: None,
            codex_auto_switch_secondary_threshold: None,
            codex_quota_alert_enabled: None,
            codex_quota_alert_primary_threshold: None,
            codex_quota_alert_secondary_threshold: None,
            codex_quota_alert_cooldown_minutes: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexInstanceStoreRef {
    #[serde(rename = "instance_id")]
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTransferExport {
    pub version: u32,
    pub export_type: String,
    #[serde(default)]
    pub account_refs: Vec<String>,
    #[serde(default)]
    pub codex_account_groups: Vec<CodexAccountGroup>,
    #[serde(default)]
    pub codex_model_providers: Vec<CodexModelProvider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_wakeup_state: Option<WakeupState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_instance_stores: Option<Vec<CodexInstanceStoreRef>>,
    #[serde(default)]
    pub current_account_refresh_map: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    pub overview_layout_mode: Option<String>,
    pub custom_sort_order: Option<Vec<String>>,
    pub local_access_entry_expanded: Option<bool>,
    pub show_code_review_quota: Option<bool>,
    pub api_switch_notice_dismissed: Option<bool>,
    pub filter_fields: Option<HashMap<String, String>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountProfile {
    pub account_id: String,
    pub email: Option<String>,
    pub plan_type: Option<String>,
    pub organization_id: Option<String>,
    pub organizations: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codex_account_oauth() {
        let data = include_str!("../../tests/fixtures/account/oauth_pro_account.json");
        let account: CodexAccount = serde_json::from_str(data).expect("parse account");
        assert!(account.is_oauth());
        assert_eq!(account.id, "acct_oauth_pro_001");
        assert_eq!(account.email, Some("developer@example.com".into()));
        assert!(account.tokens.refresh_token.is_some());
    }

    #[test]
    fn test_codex_account_apikey() {
        let data = include_str!("../../tests/fixtures/account/apikey_openai_account.json");
        let account: CodexAccount = serde_json::from_str(data).expect("parse account");
        assert!(account.is_api_key());
        assert_eq!(account.id, "acct_apikey_openai_003");
        assert!(account.api_key.is_some());
        assert_eq!(account.tokens.access_token, None);
    }

    #[test]
    fn test_account_index_load() {
        let data = include_str!("../../tests/fixtures/account/account_index.json");
        let index: CodexAccountIndex = serde_json::from_str(data).expect("parse index");
        assert_eq!(index.accounts.len(), 4);
        assert_eq!(index.current_account_id, Some("acct_oauth_pro_001".into()));
    }

    #[test]
    fn test_quota_parse() {
        let data = include_str!("../../tests/fixtures/quota/usage_normal.json");
        let quota: CodexQuota = serde_json::from_str(data).expect("parse quota");
        assert_eq!(quota.windows.len(), 2);
        assert!(quota.code_review_quota.is_some());
    }

    #[test]
    fn test_oauth_pending_state() {
        let data = include_str!("../../tests/fixtures/oauth/pending_state.json");
        let state: CodexPendingOAuthState =
            serde_json::from_str(data).expect("parse pending state");
        assert_eq!(state.login_id, "login_a1b2c3d4e5f6");
    }

    #[test]
    fn test_roundtrip_account_index() {
        let data = include_str!("../../tests/fixtures/account/account_index.json");
        let index: CodexAccountIndex = serde_json::from_str(data).expect("parse");
        let serialized = serde_json::to_string_pretty(&index).expect("serialize");
        let reparse: CodexAccountIndex = serde_json::from_str(&serialized).expect("reparse");
        assert_eq!(index.accounts.len(), reparse.accounts.len());
        assert_eq!(index.current_account_id, reparse.current_account_id);
    }

    #[test]
    fn test_roundtrip_quota() {
        let data = include_str!("../../tests/fixtures/quota/usage_near_limit.json");
        let quota: CodexQuota = serde_json::from_str(data).expect("parse");
        let serialized = serde_json::to_string_pretty(&quota).expect("serialize");
        let reparse: CodexQuota = serde_json::from_str(&serialized).expect("reparse");
        assert_eq!(quota.windows.len(), reparse.windows.len());
        assert!(quota.windows[0].percentage > 0.0);
    }

    #[test]
    fn test_local_access_collection() {
        let data = include_str!("../../tests/fixtures/local_access/collection.json");
        let collection: LocalAccessCollection =
            serde_json::from_str(data).expect("parse collection");
        assert_eq!(collection.accounts.len(), 1);
        assert!(!collection.enabled);
    }

    #[test]
    fn test_instances_load() {
        let data = include_str!("../../tests/fixtures/instances/instance_list.json");
        let instances: CodexInstanceList = serde_json::from_str(data).expect("parse instances");
        assert_eq!(instances.instances.len(), 2);
        assert!(instances.instances[0].is_default);
    }

    #[test]
    fn test_sessions_load() {
        let data = include_str!("../../tests/fixtures/sessions/session_list.json");
        let sessions: CodexSessionList = serde_json::from_str(data).expect("parse sessions");
        assert_eq!(sessions.sessions.len(), 3);
        let trashed = sessions.sessions.iter().filter(|s| s.is_trashed).count();
        assert_eq!(trashed, 1);
    }

    #[test]
    fn test_config_load() {
        let data = include_str!("../../tests/fixtures/config/default_config.json");
        let config: UserConfigCodex = serde_json::from_str(data).expect("parse config");
        assert!(config.codex_launch_on_switch.unwrap_or(false));
        assert_eq!(config.codex_auto_refresh_minutes, Some(30));
        assert_eq!(
            config
                .extra
                .get("other_provider_field")
                .and_then(|v| v.as_str()),
            Some("should_be_preserved")
        );
    }

    #[test]
    fn test_data_transfer_export() {
        let data = include_str!("../../tests/fixtures/data_transfer/export_bundle.json");
        let bundle: DataTransferExport = serde_json::from_str(data).expect("parse bundle");
        assert_eq!(bundle.export_type, "codex_only");
        assert_eq!(bundle.codex_account_groups.len(), 1);
        assert_eq!(bundle.codex_model_providers.len(), 1);
    }

    #[test]
    fn test_oauth_token_response_success() {
        let data = include_str!("../../tests/fixtures/oauth/token_response_success.json");
        let response: OAuthTokenResponse = serde_json::from_str(data).expect("parse");
        assert!(response.access_token.is_some());
        assert!(response.refresh_token.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_oauth_token_response_error() {
        let data = include_str!("../../tests/fixtures/oauth/token_response_error.json");
        let response: OAuthTokenResponse = serde_json::from_str(data).expect("parse");
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "invalid_grant");
    }

    #[test]
    fn test_empty_account_index() {
        let data = include_str!("../../tests/fixtures/account/empty_account_index.json");
        let index: CodexAccountIndex = serde_json::from_str(data).expect("parse");
        assert!(index.accounts.is_empty());
        assert!(index.current_account_id.is_none());
    }

    #[test]
    fn test_auth_file_oauth() {
        let data = include_str!("../../tests/fixtures/account/auth_file_oauth.json");
        let auth: CodexAuthFile = serde_json::from_str(data).expect("parse");
        assert!(matches!(auth.auth_mode, CodexAuthMode::OAuth));
        assert!(auth.tokens.is_some());
    }

    #[test]
    fn test_auth_file_apikey() {
        let data = include_str!("../../tests/fixtures/account/auth_file_apikey.json");
        let auth: CodexAuthFile = serde_json::from_str(data).expect("parse");
        assert!(matches!(auth.auth_mode, CodexAuthMode::ApiKey));
        assert!(auth.api_key.is_some());
    }
}
