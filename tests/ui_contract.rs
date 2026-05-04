use oauthcodex::domain::codex_models::CodexAccount;
use oauthcodex::domain::codex_models::CodexAuthMode;
use oauthcodex::domain::codex_models::CodexTokens;
use oauthcodex::domain::oauth::OAuthEvent;
use oauthcodex::domain::preferences;
use std::path::PathBuf;

#[test]
fn test_preference_key_overview_layout_mode() {
    assert_eq!(
        preferences::overview_layout_key(),
        "agtools.codex.accounts.overview_layout_mode"
    );
}

#[test]
fn test_preference_key_custom_sort_order() {
    assert_eq!(
        preferences::custom_sort_key(),
        "agtools.codex.accounts.custom_sort_order.v1"
    );
}

#[test]
fn test_preference_key_local_access_expanded() {
    assert_eq!(
        preferences::local_access_expanded_key(),
        "agtools.codex.local_access_entry_expanded.v1"
    );
}

#[test]
fn test_preference_key_code_review_quota() {
    assert_eq!(
        preferences::code_review_quota_key(),
        "agtools.codex_show_code_review_quota"
    );
}

#[test]
fn test_preference_key_api_switch_dismissed() {
    assert_eq!(
        preferences::api_switch_dismissed_key(),
        "codexApiSwitchVisibilityNoticeDismissed"
    );
}

#[test]
fn test_preference_key_current_refresh_map() {
    assert_eq!(
        preferences::current_refresh_map_key(),
        "agtools.current_account_refresh_minutes.v1"
    );
}

#[test]
fn test_preference_key_accounts_cache() {
    assert_eq!(
        preferences::accounts_cache_key(),
        "agtools.codex.accounts.cache"
    );
}

#[test]
fn test_preference_key_current_account() {
    assert_eq!(
        preferences::current_account_key(),
        "agtools.codex.accounts.current"
    );
}

#[test]
fn test_oauth_event_name_login_completed() {
    let event = OAuthEvent::LoginCompleted {
        login_id: "test".into(),
        account: Box::new(CodexAccount {
            id: "test".into(),
            provider: "codex".into(),
            auth_mode: CodexAuthMode::OAuth,
            email: None,
            plan_type: None,
            account_id: None,
            organization_id: None,
            organizations: vec![],
            display_name: "test".into(),
            tags: vec![],
            tokens: CodexTokens::empty(),
            api_key: None,
            base_url: None,
            provider_id: None,
            provider_name: None,
            api_provider_mode: None,
            quota: None,
            created_at: None,
            last_used: None,
            last_refresh: None,
        }),
    };
    assert_eq!(event.event_name(), "codex-oauth-login-completed");
}

#[test]
fn test_oauth_event_name_login_timeout() {
    let event = OAuthEvent::LoginTimeout {
        login_id: "test".into(),
    };
    assert_eq!(event.event_name(), "codex-oauth-login-timeout");
}

#[test]
fn test_runtime_file_path_auth_json() {
    let home = dirs_fake_home();
    let path = home.join(".codex").join("auth.json");
    assert!(path.to_string_lossy().ends_with(".codex/auth.json"));
}

#[test]
fn test_runtime_file_path_config_toml() {
    let home = dirs_fake_home();
    let path = home.join(".codex").join("config.toml");
    assert!(path.to_string_lossy().ends_with(".codex/config.toml"));
}

#[test]
fn test_runtime_file_path_antigravity_cockpit() {
    let home = dirs_fake_home();
    let path = home.join(".antigravity_cockpit");
    assert!(path.to_string_lossy().ends_with(".antigravity_cockpit"));
}

#[test]
fn test_runtime_file_path_cockpit_child() {
    let home = dirs_fake_home();
    let path = home
        .join(".antigravity_cockpit")
        .join("codex_account_index.json");
    assert!(path
        .to_string_lossy()
        .contains(".antigravity_cockpit/codex_account_index.json"));
}

#[test]
fn test_oauth_event_name_cancelled() {
    let event = OAuthEvent::LoginCancelled {
        login_id: "cancel_test".into(),
    };
    assert_eq!(event.event_name(), "codex-oauth-login-cancelled");
}

#[test]
fn test_oauth_event_name_error() {
    let event = OAuthEvent::LoginError {
        login_id: "err_test".into(),
        error: "Something went wrong".into(),
    };
    assert_eq!(event.event_name(), "codex-oauth-login-error");
}

fn dirs_fake_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}
