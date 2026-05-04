use oauthcodex::adapters::app_bridge::CodexAppBridge;
use oauthcodex::adapters::fs_store::CodexPaths;
use oauthcodex::domain::codex_models::CodexApiProviderMode;
use tempfile::TempDir;

fn bridge(tmp: &TempDir) -> CodexAppBridge {
    let paths = CodexPaths::for_tests(tmp.path());
    paths.ensure_dirs().expect("ensure dirs");
    CodexAppBridge::new(paths)
}

#[test]
fn app_bridge_add_switch_and_list_api_key_account() {
    let tmp = TempDir::new().expect("temp dir");
    let bridge = bridge(&tmp);

    let account = bridge
        .add_codex_account_with_api_key(
            "sk-test_bridge_key_123".into(),
            None,
            Some(CodexApiProviderMode::OpenAI),
            None,
            None,
        )
        .expect("add api key");
    let switched = bridge
        .switch_codex_account(account.id.clone())
        .expect("switch");
    let current = bridge
        .get_current_codex_account()
        .expect("current")
        .expect("some current");
    let accounts = bridge.list_codex_accounts().expect("list");

    assert_eq!(switched.id, account.id);
    assert_eq!(current.id, account.id);
    assert_eq!(accounts.len(), 1);
}

#[test]
fn app_bridge_general_config_save_accepts_ui_object_payload() {
    let tmp = TempDir::new().expect("temp dir");
    let bridge = bridge(&tmp);

    bridge
        .save_general_config(serde_json::json!({
            "codex_auto_refresh_minutes": 15,
            "codex_launch_on_switch": true
        }))
        .expect("save config");

    let config = bridge.get_general_config().expect("get config");
    assert_eq!(config["codex_auto_refresh_minutes"], 15);
    assert_eq!(config["codex_launch_on_switch"], true);
}

#[test]
fn app_bridge_local_access_commands_persist_rust_state() {
    let tmp = TempDir::new().expect("temp dir");
    let bridge = bridge(&tmp);

    let state = bridge
        .codex_local_access_update_port(7788)
        .expect("update port");
    assert_eq!(state.port, 7788);

    let enabled = bridge.codex_local_access_set_enabled(true).expect("enable");
    assert!(enabled.enabled);
}
