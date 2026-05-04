use oauthcodex::domain::codex_models::*;
use oauthcodex::domain::instance::{InstanceStore, InstanceUpdate};
use oauthcodex::domain::session::SessionManager;
use std::collections::HashMap;

fn sample_session(id: &str, instance_id: &str, trashed: bool) -> CodexSession {
    CodexSession {
        id: id.into(),
        instance_id: instance_id.into(),
        name: format!("Session {id}"),
        created_at: Some("2026-05-03T12:00:00Z".into()),
        updated_at: Some("2026-05-03T12:30:00Z".into()),
        token_count: 250,
        is_trashed: trashed,
        trash_date: if trashed {
            Some("2026-05-03T15:00:00Z".into())
        } else {
            None
        },
        model: Some("gpt-4o".into()),
        message_count: 12,
        file_path: Some(format!("/tmp/sessions/{id}.json")),
    }
}

#[test]
fn test_create_default_instance() {
    let mut store = InstanceStore::new();
    let id = store
        .create_instance("Default".into(), true, Some("/tmp/work".into()), "empty")
        .unwrap();
    assert!(id.starts_with("inst_codex_"));
    assert_eq!(store.instances().len(), 1);
    assert!(store.default_instance().is_some());
}

#[test]
fn test_create_named_instance() {
    let mut store = InstanceStore::new();
    let id = store
        .create_instance("Python".into(), false, Some("/tmp/python".into()), "empty")
        .unwrap();
    assert!(id.starts_with("inst_codex_"));
    assert_eq!(store.instances().len(), 1);
    assert!(store.default_instance().is_none());
    let inst = store.find_by_id(&id).unwrap();
    assert_eq!(inst.name, "Python");
}

#[test]
fn test_update_instance_settings() {
    let mut store = InstanceStore::new();
    let id = store
        .create_instance("Default".into(), true, None, "empty")
        .unwrap();

    let updates = InstanceUpdate {
        name: Some("Renamed".into()),
        working_dir: Some("/new/dir".into()),
        auth_mode: Some(CodexAuthMode::OAuth),
        bound_account_id: Some("acct_001".into()),
        follow_local_account: Some(false),
        launch_mode: Some(InstanceLaunchMode::Manual),
        extra_args: Some(vec!["--verbose".into()]),
        extra_env: Some(HashMap::from([("KEY".into(), "val".into())])),
    };

    store.update_instance(&id, updates).unwrap();

    let inst = store.find_by_id(&id).unwrap();
    assert_eq!(inst.name, "Renamed");
    assert_eq!(inst.working_dir, Some("/new/dir".into()));
    assert!(matches!(inst.auth_mode, Some(CodexAuthMode::OAuth)));
    assert_eq!(inst.bound_account_id, Some("acct_001".into()));
    assert!(!inst.follow_local_account);
    assert_eq!(inst.launch_mode, InstanceLaunchMode::Manual);
    assert_eq!(inst.extra_args, vec!["--verbose"]);
    assert_eq!(inst.extra_env.get("KEY").unwrap(), "val");
}

#[test]
fn test_delete_non_default_instance() {
    let mut store = InstanceStore::new();
    let id = store
        .create_instance("Custom".into(), false, None, "empty")
        .unwrap();
    store.delete_instance(&id).unwrap();
    assert!(store.instances().is_empty());
}

#[test]
fn test_cannot_delete_default_instance() {
    let mut store = InstanceStore::new();
    let id = store
        .create_instance("Default".into(), true, None, "empty")
        .unwrap();
    let result = store.delete_instance(&id);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Cannot delete default"));
}

#[test]
fn test_bind_account_to_default_instance() {
    let mut store = InstanceStore::new();
    let _id = store
        .create_instance("Default".into(), true, None, "empty")
        .unwrap();
    store.bind_account_to_default("acct_002").unwrap();
    let default = store.default_instance().unwrap();
    assert_eq!(default.bound_account_id, Some("acct_002".into()));
}

#[test]
fn test_from_list_preserves_instances() {
    let fixture_data = include_str!("fixtures/instances/instance_list.json");
    let list: CodexInstanceList = serde_json::from_str(fixture_data).expect("parse");
    let store = InstanceStore::from_list(list);
    assert_eq!(store.instances().len(), 2);
    assert!(store.default_instance().is_some());
    let default = store.default_instance().unwrap();
    assert_eq!(default.name, "Default");
    assert!(default.is_default);
    let named = store.find_by_id("inst_codex_named_001").unwrap();
    assert_eq!(named.name, "Python Project");
}

#[test]
fn test_session_manager_add_session() {
    let mut mgr = SessionManager::new();
    mgr.add_session(sample_session("sess_001", "inst_001", false))
        .unwrap();
    assert_eq!(mgr.sessions().len(), 1);
}

#[test]
fn test_session_manager_list_by_instance() {
    let mut mgr = SessionManager::new();
    mgr.add_session(sample_session("sess_a", "inst_001", false))
        .unwrap();
    mgr.add_session(sample_session("sess_b", "inst_001", false))
        .unwrap();
    mgr.add_session(sample_session("sess_c", "inst_002", false))
        .unwrap();
    mgr.add_session(sample_session("sess_d", "inst_001", true))
        .unwrap();

    let active = mgr.sessions_for_instance("inst_001");
    assert_eq!(active.len(), 2); // trashed excluded
}

#[test]
fn test_session_manager_trash_and_restore() {
    let mut mgr = SessionManager::new();
    mgr.add_session(sample_session("sess_001", "inst_001", false))
        .unwrap();

    mgr.trash_session("sess_001").unwrap();
    assert!(mgr.find_by_id("sess_001").unwrap().is_trashed);

    mgr.restore_session("sess_001").unwrap();
    assert!(!mgr.find_by_id("sess_001").unwrap().is_trashed);
}

#[test]
fn test_session_manager_token_stats() {
    let mut mgr = SessionManager::new();
    mgr.add_session(sample_session("sess_001", "inst_001", false))
        .unwrap();

    let stats = mgr.token_stats_for_session("sess_001").unwrap();
    assert_eq!(stats.total_tokens, 250);
    assert!(stats.average_per_message > 0);
    assert_eq!(stats.prompt_tokens + stats.completion_tokens, 250);
}
