use oauthcodex::adapters::fs_store::CodexPaths;
use oauthcodex::domain::account::AccountStore;
use oauthcodex::domain::codex_models::*;
use oauthcodex::domain::data_transfer::DataTransferService;
use oauthcodex::domain::group::GroupStore;
use oauthcodex::domain::instance::InstanceStore;
use oauthcodex::domain::model_provider::ModelProviderStore;
use oauthcodex::domain::wakeup::WakeupScheduler;
use std::collections::HashMap;
use tempfile::TempDir;

fn setup_account_store(tmp: &TempDir) -> AccountStore {
    let paths = CodexPaths::for_tests(tmp.path());
    let store = AccountStore::new(paths);
    store.ensure_dirs().expect("ensure dirs");
    store
}

fn setup_service(tmp: &TempDir) -> DataTransferService {
    let paths = CodexPaths::for_tests(tmp.path());
    DataTransferService::new(paths)
}

fn sample_group(id: &str, name: &str, account_ids: &[&str]) -> CodexAccountGroup {
    CodexAccountGroup {
        id: id.to_string(),
        name: name.to_string(),
        account_ids: account_ids.iter().map(|s| s.to_string()).collect(),
        sort_order: 0,
    }
}

fn sample_provider(id: &str, name: &str, base_url: &str) -> CodexModelProvider {
    CodexModelProvider {
        id: id.to_string(),
        name: name.to_string(),
        base_url: base_url.to_string(),
        api_keys: vec![],
    }
}

fn sample_oauth_account(id: &str) -> CodexAccount {
    CodexAccount {
        id: id.to_string(),
        provider: "codex".into(),
        auth_mode: CodexAuthMode::OAuth,
        email: Some(format!("{id}@example.com")),
        plan_type: Some("pro".into()),
        account_id: Some(format!("acct_{id}")),
        organization_id: None,
        organizations: vec![],
        display_name: format!("Account {id}"),
        tags: vec![],
        tokens: CodexTokens {
            access_token: Some(format!("at_{id}")),
            refresh_token: Some(format!("rt_{id}")),
            id_token: None,
            token_type: Some("Bearer".into()),
            expires_at: None,
            scope: None,
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

#[test]
fn test_export_bundle_with_account_refs_groups_providers() {
    let tmp = TempDir::new().expect("temp dir");
    let service = setup_service(&tmp);
    let account_store = setup_account_store(&tmp);

    let mut group_store = GroupStore::new();
    group_store
        .create_group("Work".into())
        .expect("create group");

    let mut provider_store = ModelProviderStore::new();
    provider_store
        .add_provider(sample_provider(
            "cmp_openai",
            "OpenAI",
            "https://api.openai.com/v1",
        ))
        .expect("add provider");

    let instance_store = InstanceStore::new();
    let wakeup_scheduler = WakeupScheduler::new();

    let bundle = service
        .export_bundle(
            &account_store,
            &group_store,
            &provider_store,
            &instance_store,
            &wakeup_scheduler,
        )
        .expect("export");

    assert_eq!(bundle.version, 1);
    assert_eq!(bundle.export_type, "codex_only");
    assert_eq!(bundle.codex_account_groups.len(), 1);
    assert_eq!(bundle.codex_model_providers.len(), 1);
}

#[test]
fn test_import_bundle_with_unresolved_account_refs() {
    let tmp = TempDir::new().expect("temp dir");
    let service = setup_service(&tmp);
    let account_store = setup_account_store(&tmp);

    let mut group_store = GroupStore::new();
    let mut provider_store = ModelProviderStore::new();

    let bundle = DataTransferExport {
        version: 1,
        export_type: "codex_only".to_string(),
        account_refs: vec!["acct_nonexistent".to_string()],
        codex_account_groups: vec![sample_group(
            "cgrp_test",
            "Test Group",
            &["acct_nonexistent"],
        )],
        codex_model_providers: vec![],
        codex_wakeup_state: None,
        codex_instance_stores: None,
        current_account_refresh_map: HashMap::new(),
    };

    let report = service
        .import_bundle(
            &account_store,
            &mut group_store,
            &mut provider_store,
            &bundle,
        )
        .expect("import");

    assert_eq!(report.unresolved_account_refs.len(), 1);
    assert!(report
        .unresolved_account_refs
        .contains(&"acct_nonexistent".to_string()));
    assert_eq!(report.imported_groups, 1);
}

#[test]
fn test_import_bundle_with_duplicate_providers() {
    let tmp = TempDir::new().expect("temp dir");
    let service = setup_service(&tmp);
    let account_store = setup_account_store(&tmp);

    let mut group_store = GroupStore::new();
    let mut provider_store = ModelProviderStore::new();

    provider_store
        .add_provider(sample_provider(
            "cmp_existing",
            "Existing",
            "https://api.test.com/v1",
        ))
        .expect("add existing");

    let bundle = DataTransferExport {
        version: 1,
        export_type: "codex_only".to_string(),
        account_refs: vec![],
        codex_account_groups: vec![],
        codex_model_providers: vec![sample_provider(
            "cmp_existing",
            "Existing",
            "https://api.test.com/v1",
        )],
        codex_wakeup_state: None,
        codex_instance_stores: None,
        current_account_refresh_map: HashMap::new(),
    };

    let report = service
        .import_bundle(
            &account_store,
            &mut group_store,
            &mut provider_store,
            &bundle,
        )
        .expect("import");

    assert_eq!(report.imported_providers, 1);
    assert!(report
        .warnings
        .iter()
        .any(|w| w.contains("Duplicate provider id")));
}

#[test]
fn test_import_bundle_with_duplicate_groups_by_name() {
    let tmp = TempDir::new().expect("temp dir");
    let service = setup_service(&tmp);
    let account_store = setup_account_store(&tmp);

    let mut group_store = GroupStore::new();
    let mut provider_store = ModelProviderStore::new();

    group_store
        .create_group("My Group".into())
        .expect("create group");

    let bundle = DataTransferExport {
        version: 1,
        export_type: "codex_only".to_string(),
        account_refs: vec![],
        codex_account_groups: vec![sample_group("cgrp_new", "My Group", &[])],
        codex_model_providers: vec![],
        codex_wakeup_state: None,
        codex_instance_stores: None,
        current_account_refresh_map: HashMap::new(),
    };

    let report = service
        .import_bundle(
            &account_store,
            &mut group_store,
            &mut provider_store,
            &bundle,
        )
        .expect("import");

    assert_eq!(report.imported_groups, 1);
    assert!(report
        .warnings
        .iter()
        .any(|w| w.contains("Group name conflict")));
}

#[test]
fn test_import_bundle_with_mixed_valid_invalid_account_refs() {
    let tmp = TempDir::new().expect("temp dir");
    let service = setup_service(&tmp);
    let account_store = setup_account_store(&tmp);

    let valid_account = sample_oauth_account("acct_valid");
    account_store
        .upsert_account(valid_account)
        .expect("add valid");

    let mut group_store = GroupStore::new();
    let mut provider_store = ModelProviderStore::new();

    let bundle = DataTransferExport {
        version: 1,
        export_type: "codex_only".to_string(),
        account_refs: vec!["acct_valid".to_string(), "acct_nonexistent".to_string()],
        codex_account_groups: vec![sample_group(
            "cgrp_mixed",
            "Mixed",
            &["acct_valid", "acct_nonexistent"],
        )],
        codex_model_providers: vec![sample_provider(
            "cmp_valid",
            "Valid Provider",
            "https://valid.example.com/v1",
        )],
        codex_wakeup_state: None,
        codex_instance_stores: None,
        current_account_refresh_map: HashMap::new(),
    };

    let report = service
        .import_bundle(
            &account_store,
            &mut group_store,
            &mut provider_store,
            &bundle,
        )
        .expect("import");

    assert!(report
        .unresolved_account_refs
        .contains(&"acct_nonexistent".to_string()));
    assert_eq!(report.imported_groups, 1);
    assert_eq!(report.imported_providers, 1);
    assert_eq!(provider_store.providers().len(), 1);
}
