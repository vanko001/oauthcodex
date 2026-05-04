use crate::adapters::http_client::HttpClient;
use crate::domain::account::AccountStore;
use crate::domain::codex_models::*;
use crate::error::CodexError;
use chrono::Utc;
use futures::stream::{self, StreamExt};
use serde::de::Error as _;
use serde::Deserialize;
use serde_json::Value;

const USAGE_API_URL: &str = "https://chatgpt.com/backend-api/wham/usage";

#[derive(Debug, Clone, Deserialize)]
struct WhamWindowInfo {
    #[serde(rename = "used_percent")]
    used_percent: Option<i32>,
    #[serde(rename = "limit_window_seconds")]
    limit_window_seconds: Option<i64>,
    #[serde(rename = "reset_after_seconds")]
    reset_after_seconds: Option<i64>,
    #[serde(rename = "reset_at")]
    reset_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
struct WhamRateLimitInfo {
    #[serde(rename = "primary_window")]
    primary_window: Option<WhamWindowInfo>,
    #[serde(rename = "secondary_window")]
    secondary_window: Option<WhamWindowInfo>,
}

#[derive(Debug, Clone, Deserialize)]
struct WhamUsageResponse {
    #[serde(rename = "plan_type")]
    plan_type: Option<String>,
    #[serde(rename = "rate_limit")]
    rate_limit: Option<WhamRateLimitInfo>,
    #[serde(rename = "code_review_rate_limit")]
    code_review_rate_limit: Option<WhamRateLimitInfo>,
}

pub struct QuotaService;

impl QuotaService {
    pub fn parse_usage_response(
        raw_json: &str,
        account_id: &str,
    ) -> Result<CodexQuota, CodexError> {
        let root: Value = serde_json::from_str(raw_json).map_err(|e| {
            CodexError::Json(serde_json::Error::custom(format!("Usage parse: {e}")))
        })?;

        if root.get("rate_limit").is_some() || root.get("code_review_rate_limit").is_some() {
            return Self::parse_wham_usage_response(root, account_id);
        }

        let plan_type = root
            .get("plan_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let windows = match root.get("windows") {
            Some(Value::Array(arr)) => arr
                .iter()
                .map(Self::parse_quota_window)
                .collect::<Result<Vec<_>, _>>()?,
            _ => vec![],
        };

        let code_review_quota = match root.get("code_review_quota") {
            Some(v) if !v.is_null() => Some(Self::parse_quota_window(v)?),
            _ => None,
        };

        let raw_data = root.get("raw_data").cloned();

        let error = root.get("error").map(|e| CodexQuotaErrorInfo {
            code: e
                .get("code")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            message: e
                .get("message")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            timestamp: e
                .get("timestamp")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            status_code: e
                .get("status_code")
                .and_then(|v| v.as_u64())
                .map(|n| n as u16),
        });

        let retry_after_ms = root.get("retry_after_ms").and_then(|v| v.as_u64());
        let resp_account_id = root
            .get("account_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let (hourly_percentage, hourly_reset_time, hourly_window_minutes, hourly_window_present) =
            Self::legacy_window_summary(&windows, "primary");
        let (weekly_percentage, weekly_reset_time, weekly_window_minutes, weekly_window_present) =
            Self::legacy_window_summary(&windows, "secondary");

        Ok(CodexQuota {
            hourly_percentage,
            hourly_reset_time,
            hourly_window_minutes,
            hourly_window_present,
            weekly_percentage,
            weekly_reset_time,
            weekly_window_minutes,
            weekly_window_present,
            account_id: resp_account_id.or(Some(account_id.to_string())),
            plan_type,
            windows,
            code_review_quota,
            error,
            retry_after_ms,
            raw_data,
        })
    }

    fn parse_wham_usage_response(root: Value, account_id: &str) -> Result<CodexQuota, CodexError> {
        let usage: WhamUsageResponse = serde_json::from_value(root.clone()).map_err(|e| {
            CodexError::Json(serde_json::Error::custom(format!("WHAM usage parse: {e}")))
        })?;

        let primary = usage
            .rate_limit
            .as_ref()
            .and_then(|r| r.primary_window.as_ref());
        let secondary = usage
            .rate_limit
            .as_ref()
            .and_then(|r| r.secondary_window.as_ref());

        let (hourly_percentage, hourly_reset_time, hourly_window_minutes) =
            Self::wham_window_summary(primary);
        let (weekly_percentage, weekly_reset_time, weekly_window_minutes) =
            Self::wham_window_summary(secondary);

        let mut windows = Vec::new();
        if let Some(primary) = primary {
            windows.push(Self::wham_window_to_legacy(
                "primary",
                "hourly",
                primary,
                hourly_percentage,
            ));
        }
        if let Some(secondary) = secondary {
            windows.push(Self::wham_window_to_legacy(
                "secondary",
                "weekly",
                secondary,
                weekly_percentage,
            ));
        }

        let code_review_quota = usage
            .code_review_rate_limit
            .as_ref()
            .and_then(|r| r.primary_window.as_ref().or(r.secondary_window.as_ref()))
            .map(|w| {
                let remaining = Self::normalize_remaining_percentage(w);
                Self::wham_window_to_legacy("code_review", "code_review", w, remaining)
            });

        Ok(CodexQuota {
            hourly_percentage,
            hourly_reset_time,
            hourly_window_minutes,
            hourly_window_present: Some(primary.is_some()),
            weekly_percentage,
            weekly_reset_time,
            weekly_window_minutes,
            weekly_window_present: Some(secondary.is_some()),
            account_id: Some(account_id.to_string()),
            plan_type: usage.plan_type,
            windows,
            code_review_quota,
            raw_data: Some(root),
            ..CodexQuota::default()
        })
    }

    fn parse_quota_window(value: &Value) -> Result<CodexQuotaWindow, CodexError> {
        let window_type = value
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let limit = value.get("limit").and_then(|v| v.as_u64()).unwrap_or(0);
        let used = value.get("used").and_then(|v| v.as_u64()).unwrap_or(0);
        let percentage = value
            .get("percentage")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let reset_at = value
            .get("reset_at")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let label = value
            .get("label")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let presence = value
            .get("presence")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(CodexQuotaWindow {
            window_type,
            limit,
            used,
            percentage,
            reset_at,
            label,
            presence,
        })
    }

    fn legacy_window_summary(
        windows: &[CodexQuotaWindow],
        window_type: &str,
    ) -> (i32, Option<i64>, Option<i64>, Option<bool>) {
        let Some(window) = windows
            .iter()
            .find(|w| w.window_type.as_deref() == Some(window_type))
        else {
            return (100, None, None, Some(false));
        };

        let remaining = (100.0 - window.percentage).round().clamp(0.0, 100.0) as i32;
        let reset = window
            .reset_at
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp());

        (remaining, reset, None, Some(true))
    }

    fn wham_window_summary(window: Option<&WhamWindowInfo>) -> (i32, Option<i64>, Option<i64>) {
        match window {
            Some(window) => (
                Self::normalize_remaining_percentage(window),
                Self::normalize_reset_time(window),
                Self::normalize_window_minutes(window),
            ),
            None => (100, None, None),
        }
    }

    fn normalize_remaining_percentage(window: &WhamWindowInfo) -> i32 {
        100 - window.used_percent.unwrap_or(0).clamp(0, 100)
    }

    fn normalize_window_minutes(window: &WhamWindowInfo) -> Option<i64> {
        let seconds = window.limit_window_seconds?;
        if seconds <= 0 {
            return None;
        }
        Some((seconds + 59) / 60)
    }

    fn normalize_reset_time(window: &WhamWindowInfo) -> Option<i64> {
        if let Some(reset_at) = window.reset_at {
            return Some(reset_at);
        }

        let reset_after_seconds = window.reset_after_seconds?;
        if reset_after_seconds < 0 {
            return None;
        }

        Some(Utc::now().timestamp() + reset_after_seconds)
    }

    fn wham_window_to_legacy(
        window_type: &str,
        label: &str,
        window: &WhamWindowInfo,
        remaining_percentage: i32,
    ) -> CodexQuotaWindow {
        let used_percent = window.used_percent.unwrap_or(0).clamp(0, 100);
        CodexQuotaWindow {
            window_type: Some(window_type.to_string()),
            limit: 100,
            used: used_percent as u64,
            percentage: remaining_percentage as f64,
            reset_at: Self::normalize_reset_time(window).map(|ts| {
                chrono::DateTime::from_timestamp(ts, 0)
                    .unwrap_or_default()
                    .to_rfc3339()
            }),
            label: Some(label.to_string()),
            presence: true,
        }
    }

    pub async fn refresh_current_quota(
        store: &AccountStore,
        http_client: &HttpClient,
    ) -> Result<Option<CodexQuota>, CodexError> {
        let account = match store.get_current_account()? {
            Some(a) => a,
            None => return Ok(None),
        };

        if account.is_api_key() {
            return Err(CodexError::Quota(
                "API key accounts do not support quota refresh".into(),
            ));
        }

        let access_token =
            account.tokens.access_token.as_ref().ok_or_else(|| {
                CodexError::InvalidState("No access token for OAuth account".into())
            })?;

        let body = http_client
            .get_usage_for_account(USAGE_API_URL, access_token, account.account_id.as_deref())
            .await?;
        let quota = Self::parse_usage_response(&body, &account.id)?;

        store.update_account_quota(&account.id, quota.clone())?;

        Ok(Some(quota))
    }

    pub async fn refresh_all_quotas(
        store: &AccountStore,
        http_client: &HttpClient,
        concurrency: usize,
    ) -> Result<Vec<(String, Result<CodexQuota, String>)>, CodexError> {
        let accounts = store.list_accounts()?;
        let oauth_accounts: Vec<CodexAccount> = accounts
            .into_iter()
            .filter(|a| a.is_oauth() && a.tokens.access_token.is_some())
            .collect();

        if oauth_accounts.is_empty() {
            return Ok(vec![]);
        }

        let concurrency = concurrency.max(1);
        let results: Vec<(String, Result<CodexQuota, String>)> = stream::iter(oauth_accounts)
            .map(|account| {
                let token = account.tokens.access_token.clone().unwrap_or_default();
                let chatgpt_account_id = account.account_id.clone();
                async move {
                    let result = async {
                        let body = http_client
                            .get_usage_for_account(
                                USAGE_API_URL,
                                &token,
                                chatgpt_account_id.as_deref(),
                            )
                            .await?;
                        QuotaService::parse_usage_response(&body, &account.id)
                    }
                    .await
                    .map_err(|e: CodexError| e.to_string());

                    (account.id, result)
                }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;

        for (account_id, result) in &results {
            if let Ok(quota) = result {
                store.update_account_quota(account_id, quota.clone())?;
            }
        }

        Ok(results)
    }

    pub fn pick_auto_switch_target(
        store: &AccountStore,
        primary_threshold: f64,
    ) -> Result<Option<String>, CodexError> {
        let accounts = store.list_accounts()?;
        let mut candidates: Vec<(String, f64)> = Vec::new();

        for account in &accounts {
            if let Some(ref quota) = account.quota {
                for window in &quota.windows {
                    if window.window_type.as_deref() == Some("primary") {
                        if window.percentage < primary_threshold {
                            candidates.push((account.id.clone(), window.percentage));
                        }
                        break;
                    }
                }
            }
        }

        if candidates.is_empty() {
            return Ok(None);
        }

        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(Some(candidates[0].0.clone()))
    }

    pub fn check_quota_alert(quota: &CodexQuota, primary_threshold: f64) -> bool {
        if quota.hourly_window_present == Some(true) || quota.weekly_window_present == Some(true) {
            return quota.hourly_percentage as f64 <= primary_threshold
                || quota.weekly_percentage as f64 <= primary_threshold;
        }

        for window in &quota.windows {
            if window.window_type.as_deref() == Some("primary") {
                return window.percentage > primary_threshold;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::fs_store::CodexPaths;
    use tempfile::TempDir;

    fn setup_test_store() -> (AccountStore, TempDir) {
        let tmp = TempDir::new().expect("temp dir");
        let paths = CodexPaths::for_tests(tmp.path());
        let store = AccountStore::new(paths);
        store.ensure_dirs().expect("ensure dirs");
        (store, tmp)
    }

    fn sample_oauth_account(id: &str, email: &str, access_token: &str) -> CodexAccount {
        CodexAccount {
            id: id.to_string(),
            provider: "codex".into(),
            auth_mode: CodexAuthMode::OAuth,
            email: Some(email.to_string()),
            plan_type: Some("pro".into()),
            account_id: Some(format!("acct_{id}")),
            organization_id: Some(format!("org_{id}")),
            organizations: vec!["developer".into()],
            display_name: format!("Account {id}"),
            tags: vec!["work".into()],
            tokens: CodexTokens {
                access_token: Some(access_token.to_string()),
                refresh_token: Some(format!("rt_{id}")),
                id_token: Some(format!("jwt_{id}")),
                token_type: Some("Bearer".into()),
                expires_at: Some("2026-05-04T00:00:00Z".into()),
                scope: Some("openid profile email".into()),
            },
            api_key: None,
            base_url: None,
            provider_id: None,
            provider_name: None,
            api_provider_mode: None,
            quota: None,
            created_at: Some("2026-05-03T12:00:00Z".into()),
            last_used: None,
            last_refresh: None,
        }
    }

    fn sample_apikey_account(id: &str) -> CodexAccount {
        CodexAccount {
            id: id.to_string(),
            provider: "codex".into(),
            auth_mode: CodexAuthMode::ApiKey,
            email: None,
            plan_type: None,
            account_id: None,
            organization_id: None,
            organizations: vec![],
            display_name: format!("API Key {id}"),
            tags: vec!["api-key".into()],
            tokens: CodexTokens::empty(),
            api_key: Some(format!("sk-test-{id}")),
            base_url: Some("https://api.openai.com/v1".into()),
            provider_id: Some("cmp_openai_default".into()),
            provider_name: Some("OpenAI".into()),
            api_provider_mode: Some(CodexApiProviderMode::OpenAI),
            quota: None,
            created_at: Some("2026-05-03T12:00:00Z".into()),
            last_used: None,
            last_refresh: None,
        }
    }

    #[test]
    fn test_parse_usage_normal() {
        let raw = include_str!("../../tests/fixtures/quota/usage_normal.json");
        let quota = QuotaService::parse_usage_response(raw, "acct_test").expect("parse quota");

        assert_eq!(quota.account_id, Some("acct_1234567890abc".into()));
        assert_eq!(quota.plan_type, Some("pro".into()));
        assert_eq!(quota.windows.len(), 2);
        assert!(quota.code_review_quota.is_some());

        let primary = quota
            .windows
            .iter()
            .find(|w| w.window_type.as_deref() == Some("primary"))
            .expect("primary window");
        assert_eq!(primary.limit, 500);
        assert_eq!(primary.used, 142);
        assert_eq!(primary.percentage, 28.4);
        assert_eq!(primary.reset_at, Some("2026-05-04T00:00:00Z".into()));
        assert_eq!(primary.label, Some("daily".into()));
        assert!(primary.presence);

        let secondary = quota
            .windows
            .iter()
            .find(|w| w.window_type.as_deref() == Some("secondary"))
            .expect("secondary window");
        assert_eq!(secondary.limit, 100);
        assert_eq!(secondary.used, 45);
        assert_eq!(secondary.percentage, 45.0);

        let review = quota.code_review_quota.as_ref().expect("code review");
        assert_eq!(review.limit, 20);
        assert_eq!(review.used, 8);
        assert_eq!(review.percentage, 40.0);

        assert!(quota.raw_data.is_some());
    }

    #[test]
    fn test_parse_usage_near_limit() {
        let raw = include_str!("../../tests/fixtures/quota/usage_near_limit.json");
        let quota = QuotaService::parse_usage_response(raw, "acct_test").expect("parse quota");

        assert_eq!(quota.plan_type, Some("free".into()));
        assert_eq!(quota.windows.len(), 1);
        assert!(quota.code_review_quota.is_none());
        assert_eq!(quota.windows[0].percentage, 96.0);
    }

    #[test]
    fn test_parse_usage_empty_windows() {
        let raw = include_str!("../../tests/fixtures/quota/usage_empty_windows.json");
        let quota = QuotaService::parse_usage_response(raw, "acct_test").expect("parse quota");

        assert!(quota.windows.is_empty());
        assert!(quota.code_review_quota.is_none());
    }

    #[test]
    fn test_parse_usage_exhausted() {
        let raw = include_str!("../../tests/fixtures/quota/usage_exhausted.json");
        let quota = QuotaService::parse_usage_response(raw, "acct_test").expect("parse quota");

        let primary = quota
            .windows
            .iter()
            .find(|w| w.window_type.as_deref() == Some("primary"))
            .expect("primary");
        assert_eq!(primary.used, 500);
        assert_eq!(primary.limit, 500);
        assert_eq!(primary.percentage, 100.0);
    }

    #[test]
    fn test_api_key_account_noop() {
        let (store, _tmp) = setup_test_store();
        let account = sample_apikey_account("apikey_001");
        store.add_account(account).expect("add");
        store
            .set_current_account("apikey_001")
            .expect("set current");

        let current = store.get_current_account().expect("get");
        assert!(current.is_some());
        assert!(current.unwrap().is_api_key());
    }

    #[test]
    fn test_no_current_account() {
        let (store, _tmp) = setup_test_store();
        let current_id = store.get_current_account_id().expect("get");
        assert!(current_id.is_none());
    }

    #[test]
    fn test_pick_auto_switch_target() {
        let (store, _tmp) = setup_test_store();

        let acct1 = sample_oauth_account("acct_001", "a@example.com", "at_001");
        let acct2 = sample_oauth_account("acct_002", "b@example.com", "at_002");
        let acct3 = sample_oauth_account("acct_003", "c@example.com", "at_003");

        store.add_account(acct1).expect("add1");
        store.add_account(acct2).expect("add2");
        store.add_account(acct3).expect("add3");

        store
            .update_account_quota(
                "acct_001",
                CodexQuota {
                    account_id: Some("acct_acct_001".into()),
                    plan_type: Some("pro".into()),
                    windows: vec![CodexQuotaWindow {
                        window_type: Some("primary".into()),
                        limit: 500,
                        used: 350,
                        percentage: 70.0,
                        reset_at: Some("2026-05-05T00:00:00Z".into()),
                        label: Some("daily".into()),
                        presence: true,
                    }],
                    code_review_quota: None,
                    error: None,
                    retry_after_ms: None,
                    raw_data: None,
                    ..CodexQuota::default()
                },
            )
            .expect("quota1");

        store
            .update_account_quota(
                "acct_002",
                CodexQuota {
                    account_id: Some("acct_acct_002".into()),
                    plan_type: Some("pro".into()),
                    windows: vec![CodexQuotaWindow {
                        window_type: Some("primary".into()),
                        limit: 500,
                        used: 50,
                        percentage: 10.0,
                        reset_at: Some("2026-05-05T00:00:00Z".into()),
                        label: Some("daily".into()),
                        presence: true,
                    }],
                    code_review_quota: None,
                    error: None,
                    retry_after_ms: None,
                    raw_data: None,
                    ..CodexQuota::default()
                },
            )
            .expect("quota2");

        store
            .update_account_quota(
                "acct_003",
                CodexQuota {
                    account_id: Some("acct_acct_003".into()),
                    plan_type: Some("pro".into()),
                    windows: vec![CodexQuotaWindow {
                        window_type: Some("primary".into()),
                        limit: 500,
                        used: 200,
                        percentage: 40.0,
                        reset_at: Some("2026-05-05T00:00:00Z".into()),
                        label: Some("daily".into()),
                        presence: true,
                    }],
                    code_review_quota: None,
                    error: None,
                    retry_after_ms: None,
                    raw_data: None,
                    ..CodexQuota::default()
                },
            )
            .expect("quota3");

        let target = QuotaService::pick_auto_switch_target(&store, 80.0)
            .expect("pick")
            .expect("should have target");
        assert_eq!(target, "acct_002");

        let target_strict = QuotaService::pick_auto_switch_target(&store, 5.0).expect("pick");
        assert!(target_strict.is_none());
    }

    #[test]
    fn test_quota_alert_threshold_exceeds() {
        let quota = CodexQuota {
            account_id: Some("acct_test".into()),
            plan_type: Some("pro".into()),
            windows: vec![CodexQuotaWindow {
                window_type: Some("primary".into()),
                limit: 500,
                used: 475,
                percentage: 95.0,
                reset_at: Some("2026-05-05T00:00:00Z".into()),
                label: Some("daily".into()),
                presence: true,
            }],
            code_review_quota: None,
            error: None,
            retry_after_ms: None,
            raw_data: None,
            ..CodexQuota::default()
        };

        assert!(QuotaService::check_quota_alert(&quota, 80.0));
        assert!(QuotaService::check_quota_alert(&quota, 90.0));
        assert!(!QuotaService::check_quota_alert(&quota, 95.0));
        assert!(!QuotaService::check_quota_alert(&quota, 99.0));
    }

    #[test]
    fn test_quota_alert_threshold_below() {
        let quota = CodexQuota {
            account_id: Some("acct_test".into()),
            plan_type: Some("pro".into()),
            windows: vec![CodexQuotaWindow {
                window_type: Some("primary".into()),
                limit: 500,
                used: 100,
                percentage: 20.0,
                reset_at: Some("2026-05-05T00:00:00Z".into()),
                label: Some("daily".into()),
                presence: true,
            }],
            code_review_quota: None,
            error: None,
            retry_after_ms: None,
            raw_data: None,
            ..CodexQuota::default()
        };

        assert!(QuotaService::check_quota_alert(&quota, 10.0));
        assert!(!QuotaService::check_quota_alert(&quota, 50.0));
    }

    #[test]
    fn test_check_quota_alert_no_primary() {
        let quota = CodexQuota {
            account_id: Some("acct_test".into()),
            plan_type: Some("pro".into()),
            windows: vec![CodexQuotaWindow {
                window_type: Some("secondary".into()),
                limit: 100,
                used: 50,
                percentage: 50.0,
                reset_at: None,
                label: None,
                presence: true,
            }],
            code_review_quota: None,
            error: None,
            retry_after_ms: None,
            raw_data: None,
            ..CodexQuota::default()
        };

        assert!(!QuotaService::check_quota_alert(&quota, 80.0));
    }

    #[test]
    fn test_pick_auto_switch_with_apikey_mixed() {
        let (store, _tmp) = setup_test_store();

        let oauth = sample_oauth_account("acct_oauth", "o@example.com", "at_o");
        let apikey = sample_apikey_account("acct_apikey");

        store.add_account(oauth).expect("add oauth");
        store.add_account(apikey).expect("add apikey");

        store
            .update_account_quota(
                "acct_oauth",
                CodexQuota {
                    account_id: Some("acct_acct_oauth".into()),
                    plan_type: Some("pro".into()),
                    windows: vec![CodexQuotaWindow {
                        window_type: Some("primary".into()),
                        limit: 500,
                        used: 300,
                        percentage: 60.0,
                        reset_at: Some("2026-05-05T00:00:00Z".into()),
                        label: Some("daily".into()),
                        presence: true,
                    }],
                    code_review_quota: None,
                    error: None,
                    retry_after_ms: None,
                    raw_data: None,
                    ..CodexQuota::default()
                },
            )
            .expect("quota oauth");

        let target = QuotaService::pick_auto_switch_target(&store, 80.0)
            .expect("pick")
            .expect("should find oauth");
        assert_eq!(target, "acct_oauth");
    }

    #[test]
    fn test_roundtrip_parse_usage() {
        let raw = include_str!("../../tests/fixtures/quota/usage_normal.json");
        let quota = QuotaService::parse_usage_response(raw, "acct_test").expect("parse");

        let serialized = serde_json::to_string_pretty(&quota).expect("serialize");
        let reparse: CodexQuota = serde_json::from_str(&serialized).expect("reparse");

        assert_eq!(quota.windows.len(), reparse.windows.len());
        assert_eq!(quota.plan_type, reparse.plan_type);
        assert_eq!(quota.account_id, reparse.account_id);
    }
}
