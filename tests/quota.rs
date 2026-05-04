use oauthcodex::adapters::fs_store::CodexPaths;
use oauthcodex::domain::account::AccountStore;
use oauthcodex::domain::codex_models::*;
use oauthcodex::domain::quota::QuotaService;
use tempfile::TempDir;

fn setup_store(tmp: &TempDir) -> AccountStore {
    let paths = CodexPaths::for_tests(tmp.path());
    let store = AccountStore::new(paths);
    store.ensure_dirs().expect("ensure dirs");
    store
}

fn sample_oauth(id: &str, access_token: &str) -> CodexAccount {
    CodexAccount {
        id: id.to_string(),
        provider: "codex".into(),
        auth_mode: CodexAuthMode::OAuth,
        email: Some(format!("{id}@example.com")),
        plan_type: Some("pro".into()),
        account_id: Some(format!("acct_{id}")),
        organization_id: Some(format!("org_{id}")),
        organizations: vec!["developer".into()],
        display_name: format!("Account {id}"),
        tags: vec![],
        tokens: CodexTokens {
            access_token: Some(access_token.to_string()),
            refresh_token: Some(format!("rt_{id}")),
            id_token: Some(format!("jwt_{id}")),
            token_type: Some("Bearer".into()),
            expires_at: Some("2026-05-05T00:00:00Z".into()),
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

fn sample_apikey(id: &str) -> CodexAccount {
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
    let raw = include_str!("fixtures/quota/usage_normal.json");
    let quota = QuotaService::parse_usage_response(raw, "acct_test").expect("parse");

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

    let code_review = quota.code_review_quota.as_ref().expect("code review");
    assert_eq!(code_review.limit, 20);
    assert_eq!(code_review.used, 8);
    assert_eq!(code_review.percentage, 40.0);
    assert!(quota.raw_data.is_some());
}

#[test]
fn test_parse_usage_near_limit() {
    let raw = include_str!("fixtures/quota/usage_near_limit.json");
    let quota = QuotaService::parse_usage_response(raw, "acct_test").expect("parse");

    assert_eq!(quota.plan_type, Some("free".into()));
    assert_eq!(quota.windows.len(), 1);
    assert_eq!(quota.windows[0].percentage, 96.0);
    assert_eq!(quota.windows[0].limit, 50);
    assert_eq!(quota.windows[0].used, 48);
    assert!(quota.code_review_quota.is_none());
}

#[test]
fn test_parse_usage_exhausted() {
    let raw = include_str!("fixtures/quota/usage_exhausted.json");
    let quota = QuotaService::parse_usage_response(raw, "acct_test").expect("parse");

    let primary = quota
        .windows
        .iter()
        .find(|w| w.window_type.as_deref() == Some("primary"))
        .expect("primary");
    assert_eq!(primary.limit, 500);
    assert_eq!(primary.used, 500);
    assert_eq!(primary.percentage, 100.0);

    let code_review = quota.code_review_quota.as_ref().expect("code review");
    assert_eq!(code_review.used, 20);
    assert_eq!(code_review.limit, 20);
    assert_eq!(code_review.percentage, 100.0);
}

#[test]
fn test_parse_usage_error_401() {
    let raw = include_str!("fixtures/quota/usage_error_401.json");
    let quota = QuotaService::parse_usage_response(raw, "acct_test").expect("parse");

    let error = quota.error.as_ref().expect("error field present");
    assert_eq!(error.code.as_deref(), Some("auth_error"));
    assert_eq!(error.message.as_deref(), Some("Invalid or expired token"));
    assert_eq!(error.timestamp.as_deref(), Some("2026-05-03T12:00:00Z"));
    assert_eq!(error.status_code, Some(401));
    assert!(quota.windows.is_empty());
}

#[test]
fn test_parse_usage_error_403() {
    let raw = include_str!("fixtures/quota/usage_error_403.json");
    let quota = QuotaService::parse_usage_response(raw, "acct_test").expect("parse");

    let error = quota.error.as_ref().expect("error field present");
    assert_eq!(error.code.as_deref(), Some("forbidden"));
    assert_eq!(error.message.as_deref(), Some("Access denied to usage API"));
    assert_eq!(error.status_code, Some(403));
}

#[test]
fn test_parse_usage_error_429_retry_after() {
    let raw = include_str!("fixtures/quota/usage_error_429_retry_after.json");
    let quota = QuotaService::parse_usage_response(raw, "acct_test").expect("parse");

    let error = quota.error.as_ref().expect("error field present");
    assert_eq!(error.code.as_deref(), Some("rate_limited"));
    assert_eq!(error.status_code, Some(429));
    assert_eq!(quota.retry_after_ms, Some(5000));
}

#[test]
fn test_api_key_account_noop_refresh() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    let account = sample_apikey("apikey_001");
    store.add_account(account).expect("add");
    store
        .set_current_account("apikey_001")
        .expect("set current");

    let current = store.get_current_account().expect("get");
    assert!(current.is_some());
    assert!(current.unwrap().is_api_key());
}

#[test]
fn test_pick_auto_switch_target_three_accounts() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    store
        .add_account(sample_oauth("acct_a", "at_a"))
        .expect("add a");
    store
        .add_account(sample_oauth("acct_b", "at_b"))
        .expect("add b");
    store
        .add_account(sample_oauth("acct_c", "at_c"))
        .expect("add c");

    store
        .update_account_quota(
            "acct_a",
            CodexQuota {
                account_id: Some("acct_acct_a".into()),
                plan_type: Some("pro".into()),
                windows: vec![CodexQuotaWindow {
                    window_type: Some("primary".into()),
                    limit: 500,
                    used: 450,
                    percentage: 90.0,
                    reset_at: Some("2026-05-05T00:00:00Z".into()),
                    label: Some("daily".into()),
                    presence: true,
                }],
                code_review_quota: None,
                error: None,
                retry_after_ms: None,
                raw_data: None,
            },
        )
        .expect("quota a");

    store
        .update_account_quota(
            "acct_b",
            CodexQuota {
                account_id: Some("acct_acct_b".into()),
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
            },
        )
        .expect("quota b");

    store
        .update_account_quota(
            "acct_c",
            CodexQuota {
                account_id: Some("acct_acct_c".into()),
                plan_type: Some("pro".into()),
                windows: vec![CodexQuotaWindow {
                    window_type: Some("primary".into()),
                    limit: 500,
                    used: 250,
                    percentage: 50.0,
                    reset_at: Some("2026-05-05T00:00:00Z".into()),
                    label: Some("daily".into()),
                    presence: true,
                }],
                code_review_quota: None,
                error: None,
                retry_after_ms: None,
                raw_data: None,
            },
        )
        .expect("quota c");

    let target = QuotaService::pick_auto_switch_target(&store, 80.0)
        .expect("pick")
        .expect("should find target");
    assert_eq!(target, "acct_b");
}

#[test]
fn test_quota_alert_threshold() {
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
    };

    assert!(QuotaService::check_quota_alert(&quota, 80.0));
    assert!(!QuotaService::check_quota_alert(&quota, 95.0));
}

#[test]
fn test_roundtrip_parse_fixture_serialize_reparse() {
    let raw = include_str!("fixtures/quota/usage_normal.json");
    let quota = QuotaService::parse_usage_response(raw, "acct_test").expect("parse");

    let serialized = serde_json::to_string_pretty(&quota).expect("serialize");
    let reparse: CodexQuota = serde_json::from_str(&serialized).expect("reparse");

    assert_eq!(quota.windows.len(), reparse.windows.len());
    assert_eq!(quota.plan_type, reparse.plan_type);
    assert_eq!(quota.account_id, reparse.account_id);
    assert_eq!(quota.retry_after_ms, reparse.retry_after_ms);
}
