use oauthcodex::adapters::fs_store::CodexPaths;
use oauthcodex::domain::codex_models::UserConfigCodex;
use oauthcodex::domain::config::ConfigStore;
use std::collections::HashMap;
use tempfile::TempDir;

fn setup(tmp: &TempDir) -> ConfigStore {
    let paths = CodexPaths::for_tests(tmp.path());
    ConfigStore::new(paths)
}

#[test]
fn test_load_default_config() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup(&tmp);
    let config = store.load_config().expect("load config");
    assert_eq!(config.codex_auto_refresh_minutes, Some(30));
    assert!(config.extra.is_empty());
}

#[test]
fn test_set_codex_auto_refresh_valid() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup(&tmp);
    store.set_codex_auto_refresh(60).expect("set refresh");
    let config = store.load_config().expect("load config");
    assert_eq!(config.codex_auto_refresh_minutes, Some(60));
}

#[test]
fn test_set_codex_auto_refresh_negative_one() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup(&tmp);
    store.set_codex_auto_refresh(-1).expect("set refresh -1");
    let config = store.load_config().expect("load config");
    assert_eq!(config.codex_auto_refresh_minutes, Some(-1));
}

#[test]
fn test_set_codex_auto_refresh_zero_clamped_to_one() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup(&tmp);
    store.set_codex_auto_refresh(0).expect("set refresh 0");
    let config = store.load_config().expect("load config");
    assert_eq!(config.codex_auto_refresh_minutes, Some(1));
}

#[test]
fn test_set_codex_auto_refresh_9999_clamped_to_1440() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup(&tmp);
    store
        .set_codex_auto_refresh(9999)
        .expect("set refresh 9999");
    let config = store.load_config().expect("load config");
    assert_eq!(config.codex_auto_refresh_minutes, Some(1440));
}

#[test]
fn test_set_launch_on_switch() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup(&tmp);
    store.set_codex_launch_on_switch(true).expect("set true");
    let config = store.load_config().expect("load config");
    assert_eq!(config.codex_launch_on_switch, Some(true));

    store.set_codex_launch_on_switch(false).expect("set false");
    let config = store.load_config().expect("load config");
    assert_eq!(config.codex_launch_on_switch, Some(false));
}

#[test]
fn test_set_local_access_entry_visible() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup(&tmp);
    store
        .set_codex_local_access_entry_visible(true)
        .expect("set visible");
    let config = store.load_config().expect("load config");
    assert_eq!(config.codex_local_access_entry_visible, Some(true));
}

#[test]
fn test_set_app_path() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup(&tmp);
    store
        .set_codex_app_path("/usr/local/bin/codex")
        .expect("set path");
    let config = store.load_config().expect("load config");
    assert_eq!(
        config.codex_app_path,
        Some("/usr/local/bin/codex".to_string())
    );

    store.set_codex_app_path("").expect("clear path");
    let config = store.load_config().expect("load config");
    assert_eq!(config.codex_app_path, None);
}

#[test]
fn test_set_auto_switch_valid_thresholds() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup(&tmp);
    store
        .set_codex_auto_switch(true, Some(80), Some(20))
        .expect("set");
    let config = store.load_config().expect("load config");
    assert_eq!(config.codex_auto_switch_enabled, Some(true));
    assert_eq!(config.codex_auto_switch_primary_threshold, Some(80));
    assert_eq!(config.codex_auto_switch_secondary_threshold, Some(20));
}

#[test]
fn test_set_auto_switch_negative_clamped_to_0() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup(&tmp);
    store
        .set_codex_auto_switch(true, Some(-5), Some(-10))
        .expect("set");
    let config = store.load_config().expect("load config");
    assert_eq!(config.codex_auto_switch_primary_threshold, Some(0));
    assert_eq!(config.codex_auto_switch_secondary_threshold, Some(0));
}

#[test]
fn test_set_auto_switch_over_100_clamped_to_100() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup(&tmp);
    store
        .set_codex_auto_switch(true, Some(200), Some(999))
        .expect("set");
    let config = store.load_config().expect("load config");
    assert_eq!(config.codex_auto_switch_primary_threshold, Some(100));
    assert_eq!(config.codex_auto_switch_secondary_threshold, Some(100));
}

#[test]
fn test_set_quota_alert_thresholds() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup(&tmp);
    store
        .set_codex_quota_alert(true, Some(75))
        .expect("set alert");
    let config = store.load_config().expect("load config");
    assert_eq!(config.codex_quota_alert_enabled, Some(true));
    assert_eq!(config.codex_quota_alert_primary_threshold, Some(75));
}

#[test]
fn test_set_quota_alert_clamping() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup(&tmp);
    store
        .set_codex_quota_alert(true, Some(150))
        .expect("set alert");
    let config = store.load_config().expect("load config");
    assert_eq!(config.codex_quota_alert_primary_threshold, Some(100));
}

#[test]
fn test_roundtrip_with_extra_fields_preserved() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup(&tmp);

    let mut config = UserConfigCodex {
        codex_auto_refresh_minutes: Some(45),
        codex_launch_on_switch: Some(true),
        ..Default::default()
    };

    let mut extra = HashMap::new();
    extra.insert(
        "other_provider_field".to_string(),
        serde_json::Value::String("should_be_preserved".to_string()),
    );
    extra.insert("custom_flag".to_string(), serde_json::Value::Bool(true));
    config.extra = extra;

    store.save_config(&config).expect("save config");

    let loaded = store.load_config().expect("load config");
    assert_eq!(loaded.codex_auto_refresh_minutes, Some(45));
    assert_eq!(loaded.codex_launch_on_switch, Some(true));
    assert_eq!(
        loaded
            .extra
            .get("other_provider_field")
            .and_then(|v| v.as_str()),
        Some("should_be_preserved")
    );
    assert_eq!(
        loaded.extra.get("custom_flag").and_then(|v| v.as_bool()),
        Some(true)
    );
}
