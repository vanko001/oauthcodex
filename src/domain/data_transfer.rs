use crate::adapters::fs_store::CodexPaths;
use crate::domain::account::AccountStore;
use crate::domain::codex_models::*;
use crate::domain::group::GroupStore;
use crate::domain::instance::InstanceStore;
use crate::domain::model_provider::ModelProviderStore;
use crate::domain::wakeup::WakeupScheduler;
use crate::error::CodexError;

#[derive(Debug, Clone)]
pub struct ImportReport {
    pub unresolved_account_refs: Vec<String>,
    pub imported_groups: usize,
    pub imported_providers: usize,
    pub warnings: Vec<String>,
}

pub struct DataTransferService {
    pub paths: CodexPaths,
}

impl DataTransferService {
    pub fn new(paths: CodexPaths) -> Self {
        Self { paths }
    }

    pub fn export_bundle(
        &self,
        _account_store: &AccountStore,
        group_store: &GroupStore,
        provider_store: &ModelProviderStore,
        instance_store: &InstanceStore,
        wakeup_scheduler: &WakeupScheduler,
    ) -> Result<DataTransferExport, CodexError> {
        let mut account_refs: Vec<String> = Vec::new();

        for group in group_store.groups() {
            for account_id in &group.account_ids {
                if !account_refs.contains(account_id) {
                    account_refs.push(account_id.clone());
                }
            }
        }

        let codex_account_groups = group_store.groups().to_vec();

        let codex_model_providers = provider_store.providers().to_vec();

        let codex_wakeup_state = wakeup_scheduler.to_state();

        let codex_instance_stores: Option<Vec<CodexInstanceStoreRef>> = {
            let instances = instance_store.instances();
            if instances.is_empty() {
                None
            } else {
                Some(
                    instances
                        .iter()
                        .map(|i| CodexInstanceStoreRef {
                            id: i.id.clone(),
                            name: i.name.clone(),
                            is_default: i.is_default,
                        })
                        .collect(),
                )
            }
        };

        let current_account_refresh_map = wakeup_scheduler.current_refresh_map();

        Ok(DataTransferExport {
            version: 1,
            export_type: "codex_only".to_string(),
            account_refs,
            codex_account_groups,
            codex_model_providers,
            codex_wakeup_state,
            codex_instance_stores,
            current_account_refresh_map,
        })
    }

    pub fn import_bundle(
        &self,
        account_store: &AccountStore,
        group_store: &mut GroupStore,
        provider_store: &mut ModelProviderStore,
        bundle: &DataTransferExport,
    ) -> Result<ImportReport, CodexError> {
        let mut unresolved_account_refs = Vec::new();
        let mut warnings = Vec::new();

        for account_id in &bundle.account_refs {
            if !account_ref_exists(account_store, account_id) {
                unresolved_account_refs.push(account_id.clone());
            }
        }

        for group in &bundle.codex_account_groups {
            if group_store.find_by_id(&group.id).is_some() {
                warnings.push(format!(
                    "Duplicate group id: {} (name: {})",
                    group.id, group.name
                ));
            } else {
                let group_name_lower = group.name.to_lowercase();
                let name_exists = group_store
                    .groups()
                    .iter()
                    .any(|g| g.name.to_lowercase() == group_name_lower);
                if name_exists {
                    warnings.push(format!(
                        "Group name conflict: {} (id: {})",
                        group.name, group.id
                    ));
                }
                group_store.import_group(group.clone());

                for account_id in &group.account_ids {
                    if !account_ref_exists(account_store, account_id)
                        && !unresolved_account_refs.contains(account_id)
                    {
                        unresolved_account_refs.push(account_id.clone());
                    }
                }
            }
        }

        for provider in &bundle.codex_model_providers {
            if provider_store.find_by_id(&provider.id).is_some() {
                warnings.push(format!(
                    "Duplicate provider id: {} (name: {})",
                    provider.id, provider.name
                ));
            } else {
                let base_url_lower = provider.base_url.to_lowercase();
                let url_exists = provider_store
                    .providers()
                    .iter()
                    .any(|p| p.base_url.to_lowercase() == base_url_lower);
                if url_exists {
                    warnings.push(format!(
                        "Provider base URL conflict: {} (id: {})",
                        provider.base_url, provider.id
                    ));
                }
                match provider_store.import_provider(provider.clone()) {
                    Ok(()) => {}
                    Err(e) => {
                        warnings.push(format!("Failed to import provider {}: {}", provider.id, e));
                    }
                }
            }
        }

        Ok(ImportReport {
            unresolved_account_refs,
            imported_groups: bundle.codex_account_groups.len(),
            imported_providers: bundle.codex_model_providers.len(),
            warnings,
        })
    }
}

fn account_ref_exists(account_store: &AccountStore, account_id: &str) -> bool {
    account_store
        .load_account_file_opt(account_id)
        .ok()
        .flatten()
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::fs_store::CodexPaths;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn setup_test_env() -> (DataTransferService, TempDir) {
        let tmp = TempDir::new().expect("temp dir");
        let paths = CodexPaths::for_tests(tmp.path());
        let service = DataTransferService::new(paths);
        (service, tmp)
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

    fn mock_wakeup_scheduler() -> WakeupScheduler {
        WakeupScheduler::new()
    }

    #[test]
    fn test_import_bundle_with_unresolved_refs() {
        let (service, tmp) = setup_test_env();

        let account_paths = CodexPaths::for_tests(tmp.path());
        let account_store = AccountStore::new(account_paths);
        account_store.ensure_dirs().expect("ensure dirs");

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
            .expect("import bundle");

        assert_eq!(report.unresolved_account_refs.len(), 1);
        assert!(report
            .unresolved_account_refs
            .contains(&"acct_nonexistent".to_string()));
        assert_eq!(report.imported_groups, 1);
    }

    #[test]
    fn test_import_bundle_with_duplicate_providers() {
        let (service, tmp) = setup_test_env();

        let account_paths = CodexPaths::for_tests(tmp.path());
        let account_store = AccountStore::new(account_paths);
        account_store.ensure_dirs().expect("ensure dirs");

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
            .expect("import bundle");

        assert_eq!(report.imported_providers, 1);
        assert!(report
            .warnings
            .iter()
            .any(|w| w.contains("Duplicate provider id")));
    }

    #[test]
    fn test_import_bundle_with_duplicate_groups() {
        let (service, tmp) = setup_test_env();

        let account_paths = CodexPaths::for_tests(tmp.path());
        let account_store = AccountStore::new(account_paths);
        account_store.ensure_dirs().expect("ensure dirs");

        let mut group_store = GroupStore::new();
        let mut provider_store = ModelProviderStore::new();

        group_store
            .create_group("My Group".to_string())
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
            .expect("import bundle");

        assert_eq!(report.imported_groups, 1);
        assert!(report
            .warnings
            .iter()
            .any(|w| w.contains("Group name conflict")));
    }

    #[test]
    fn test_export_bundle() {
        let (service, tmp) = setup_test_env();

        let account_paths = CodexPaths::for_tests(tmp.path());
        let account_store = AccountStore::new(account_paths);
        account_store.ensure_dirs().expect("ensure dirs");

        let mut group_store = GroupStore::new();
        group_store
            .create_group("Work".to_string())
            .expect("create group");

        let provider_store = ModelProviderStore::new();

        let instance_store = InstanceStore::new();
        let wakeup_scheduler = mock_wakeup_scheduler();

        let bundle = service
            .export_bundle(
                &account_store,
                &group_store,
                &provider_store,
                &instance_store,
                &wakeup_scheduler,
            )
            .expect("export bundle");

        assert_eq!(bundle.version, 1);
        assert_eq!(bundle.export_type, "codex_only");
        assert_eq!(bundle.codex_account_groups.len(), 1);
    }

    #[test]
    fn test_export_bundle_with_empty_collections() {
        let (service, tmp) = setup_test_env();

        let account_paths = CodexPaths::for_tests(tmp.path());
        let account_store = AccountStore::new(account_paths);
        account_store.ensure_dirs().expect("ensure dirs");

        let group_store = GroupStore::new();
        let provider_store = ModelProviderStore::new();
        let instance_store = InstanceStore::new();
        let wakeup_scheduler = mock_wakeup_scheduler();

        let bundle = service
            .export_bundle(
                &account_store,
                &group_store,
                &provider_store,
                &instance_store,
                &wakeup_scheduler,
            )
            .expect("export bundle");

        assert_eq!(bundle.version, 1);
        assert_eq!(bundle.export_type, "codex_only");
        assert!(bundle.account_refs.is_empty());
        assert!(bundle.codex_account_groups.is_empty());
        assert!(bundle.codex_model_providers.is_empty());
        assert!(bundle.codex_instance_stores.is_none());
    }
}
