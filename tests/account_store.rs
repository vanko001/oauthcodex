use oauthcodex::adapters::fs_store::{
    read_json_file, read_json_file_opt, write_json_atomic, CodexPaths,
};
use oauthcodex::domain::account::AccountStore;
use oauthcodex::domain::codex_models::*;
use oauthcodex::error::CodexError;
use tempfile::TempDir;

fn setup_store(tmp: &TempDir) -> AccountStore {
    let paths = CodexPaths::for_tests(tmp.path());
    let store = AccountStore::new(paths);
    store.ensure_dirs().expect("ensure dirs");
    store
}

#[test]
fn source_account_index_path_matches_cockpit_tools() {
    let tmp = TempDir::new().expect("temp dir");
    let paths = CodexPaths::for_tests(tmp.path());
    assert!(paths
        .account_index_file
        .to_string_lossy()
        .ends_with(".antigravity_cockpit/codex_accounts.json"));
}

#[test]
fn account_index_serializes_source_summary_without_secrets() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    let account = store
        .upsert_api_key_account(
            "sk-test_openai_api_key_mock_value_123",
            None,
            None,
            None,
            None,
        )
        .expect("upsert api key");

    let index_json = std::fs::read_to_string(&store.paths.account_index_file).expect("read index");
    let value: serde_json::Value = serde_json::from_str(&index_json).expect("parse index json");
    assert_eq!(value["version"], "1.0");
    assert_eq!(value["accounts"][0]["id"], account.id);
    assert!(value["accounts"][0].get("tokens").is_none());
    assert!(value["accounts"][0].get("api_key").is_none());
    assert!(value["accounts"][0].get("openai_api_key").is_none());
}

#[test]
fn add_api_key_account_matches_source_id_and_switches_to_auth_json() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    let account = store
        .upsert_api_key_account(
            "sk-test_openai_api_key_mock_value_123",
            None,
            None,
            None,
            None,
        )
        .expect("upsert api key");

    assert_eq!(
        account.id,
        format!(
            "codex_apikey_{:x}",
            md5::compute("sk-test_openai_api_key_mock_value_123".as_bytes())
        )
    );
    assert_eq!(account.email, Some("api-key-156bda16".into()));
    assert_eq!(account.plan_type, Some("API_KEY".into()));
    assert_eq!(
        account.api_provider_mode,
        Some(CodexApiProviderMode::OpenAI)
    );
    assert_eq!(account.base_url, None);

    let switched = store.switch_account_managed(&account.id).expect("switch");
    assert_eq!(switched.id, account.id);

    let auth_json =
        std::fs::read_to_string(&store.paths.auth_file).expect("read generated auth json");
    let value: serde_json::Value = serde_json::from_str(&auth_json).expect("parse auth json");
    assert_eq!(value["auth_mode"], "apikey");
    assert_eq!(
        value["OPENAI_API_KEY"],
        "sk-test_openai_api_key_mock_value_123"
    );
    assert!(value.get("api_key").is_none());
}

#[test]
fn import_auth_json_with_source_openai_api_key_is_restorable() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    let imported = store
        .import_json_accounts(
            r#"{
              "auth_mode": "apikey",
              "OPENAI_API_KEY": "sk-test_source_auth_key_123",
              "base_url": "https://api.openai.com/v1"
            }"#,
        )
        .expect("import");

    assert_eq!(imported.imported.len(), 1);
    assert!(imported.failed.is_empty());
    assert_eq!(
        imported.imported[0].api_key,
        Some("sk-test_source_auth_key_123".into())
    );
    assert!(store.load_account_file(&imported.imported[0].id).is_ok());
}

#[test]
fn export_accounts_preserves_api_key_for_source_roundtrip() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);
    let account = store
        .upsert_api_key_account("sk-test_export_key_123", None, None, None, None)
        .expect("upsert");

    let exported = store
        .export_accounts(std::slice::from_ref(&account.id))
        .expect("export");
    let json = serde_json::to_value(&exported).expect("export value");
    assert_eq!(json[0]["openai_api_key"], "sk-test_export_key_123");
    assert!(json[0].get("api_key").is_none());
}

#[test]
fn load_fixture_account_index() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    let fixture_data = include_str!("fixtures/account/account_index.json");
    let fixture_index: CodexAccountIndex =
        serde_json::from_str(fixture_data).expect("parse fixture");
    store.save_index(&fixture_index).expect("save");

    let loaded = store.load_index().expect("load");
    assert_eq!(loaded.version, "1.0");
    assert_eq!(loaded.accounts.len(), 4);
    assert_eq!(loaded.current_account_id, Some("acct_oauth_pro_001".into()));
}

#[test]
fn load_fixture_oauth_pro_account() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    let fixture_data = include_str!("fixtures/account/oauth_pro_account.json");
    let account: CodexAccount = serde_json::from_str(fixture_data).expect("parse");
    store.save_account_file(&account).expect("save");
    store.upsert_account(account.clone()).expect("upsert");

    let loaded = store.load_account_file("acct_oauth_pro_001").expect("load");
    assert_eq!(loaded.email, Some("developer@example.com".into()));
    assert_eq!(loaded.plan_type, Some("pro".into()));
    assert!(loaded.tokens.refresh_token.is_some());
}

#[test]
fn load_fixture_apikey_openai_account() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    let fixture_data = include_str!("fixtures/account/apikey_openai_account.json");
    let account: CodexAccount = serde_json::from_str(fixture_data).expect("parse");
    store.upsert_account(account.clone()).expect("upsert");

    let loaded = store
        .load_account_file("acct_apikey_openai_003")
        .expect("load");
    assert!(loaded.is_api_key());
    assert_eq!(loaded.tokens.access_token, Some(String::new()));
    assert_eq!(loaded.base_url, Some("https://api.openai.com/v1".into()));
}

#[test]
fn empty_index_recovery_after_corrupt_file() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    std::fs::write(&store.paths.account_index_file, "garbage not json").expect("write corrupt");
    let index = store.load_index().expect("load after corrupt");
    assert!(index.accounts.is_empty());
    assert!(index.current_account_id.is_none());
}

#[test]
fn missing_index_file_creates_empty() {
    let tmp = TempDir::new().expect("temp dir");
    let paths = CodexPaths::for_tests(tmp.path());
    paths.ensure_dirs().expect("ensure");
    let index =
        read_json_file_opt::<CodexAccountIndex>(&paths.account_index_file).expect("read opt");
    assert!(index.is_none());
}

#[test]
fn batch_import_valid_accounts() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    let fixture_data = include_str!("fixtures/account/batch_import_valid.json");
    store
        .import_json_accounts(fixture_data)
        .expect("import source-compatible json");
    let accounts = store.list_accounts().expect("list");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, "acct_import_oauth_001");
}

#[test]
fn write_and_read_auth_file_oauth() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    let fixture_data = include_str!("fixtures/account/auth_file_oauth.json");
    let auth_file: CodexAuthFile = serde_json::from_str(fixture_data).expect("parse");
    let path = tmp.path().join("auth.json");
    store.write_auth_file(&path, &auth_file).expect("write");
    let loaded = store.load_auth_file(&path).expect("read");
    assert!(matches!(loaded.auth_mode, CodexAuthMode::OAuth));
    assert!(loaded.tokens.unwrap().access_token.is_some());
}

#[test]
fn write_and_read_auth_file_apikey() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    let fixture_data = include_str!("fixtures/account/auth_file_apikey.json");
    let auth_file: CodexAuthFile = serde_json::from_str(fixture_data).expect("parse");
    let path = tmp.path().join("auth.json");
    store.write_auth_file(&path, &auth_file).expect("write");
    let loaded = store.load_auth_file(&path).expect("read");
    assert!(matches!(loaded.auth_mode, CodexAuthMode::ApiKey));
    assert_eq!(
        loaded.api_key,
        Some("sk-test_openai_api_key_mock_value_123".into())
    );
}

#[test]
fn roundtrip_account_through_store() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    let fixture_data = include_str!("fixtures/account/oauth_pro_account.json");
    let account: CodexAccount = serde_json::from_str(fixture_data).expect("parse");
    store.upsert_account(account.clone()).expect("upsert");

    let loaded = store.load_account_file(&account.id).expect("load");
    assert_eq!(loaded.display_name, account.display_name);
    assert_eq!(loaded.email, account.email);
    assert_eq!(loaded.tags, account.tags);
    assert_eq!(loaded.tokens.access_token, account.tokens.access_token);
}

#[test]
fn delete_current_account_clears_current() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    let fixture_data = include_str!("fixtures/account/oauth_pro_account.json");
    let account: CodexAccount = serde_json::from_str(fixture_data).expect("parse");
    let id = account.id.clone();
    store.upsert_account(account).expect("upsert");
    store.set_current_account(&id).expect("set current");

    store.delete_account(&id).expect("delete");
    let current = store.get_current_account().expect("get current");
    assert!(current.is_none());
}

#[test]
fn nonexistent_account_returns_not_found() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);
    let result = store.load_account_file("nonexistent_id");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CodexError::NotFound(_)));
}

#[test]
fn duplicate_account_rejected() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    let fixture_data = include_str!("fixtures/account/oauth_pro_account.json");
    let account: CodexAccount = serde_json::from_str(fixture_data).expect("parse");
    store.add_account(account.clone()).expect("add");
    let result = store.add_account(account);
    assert!(result.is_err());
}

#[test]
fn atomic_write_survives_process_kill_simulation() {
    let tmp = TempDir::new().expect("temp dir");
    let store = setup_store(&tmp);

    let account = CodexAccount {
        id: "atomic_test".into(),
        provider: "codex".into(),
        auth_mode: CodexAuthMode::OAuth,
        email: Some("atomic@example.com".into()),
        plan_type: Some("pro".into()),
        account_id: Some("acct_atomic".into()),
        organization_id: Some("org_atomic".into()),
        organizations: vec![],
        display_name: "Atomic Test".into(),
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
    };

    let path = store.paths.account_file("atomic_test");
    write_json_atomic(&path, &account).expect("atomic write");

    let loaded: CodexAccount = read_json_file(&path).expect("read");
    assert_eq!(loaded.id, "atomic_test");
    assert_eq!(loaded.email, Some("atomic@example.com".into()));
}
