use crate::adapters::fs_store::{read_json_file_opt, write_json_atomic, CodexPaths};
use crate::domain::account::{AccountStore, ImportResult};
use crate::domain::codex_models::*;
use crate::domain::config::ConfigStore;
use crate::domain::group::GroupStore;
use crate::domain::local_access::LocalAccessService;
use crate::domain::model_provider::ModelProviderStore;
use crate::domain::oauth::OAuthService;
use crate::error::CodexError;

pub struct CodexAppBridge {
    pub paths: CodexPaths,
}

impl CodexAppBridge {
    pub fn new(paths: CodexPaths) -> Self {
        Self { paths }
    }

    fn account_store(&self) -> AccountStore {
        AccountStore::new(self.paths.clone())
    }

    fn config_store(&self) -> ConfigStore {
        ConfigStore::new(self.paths.clone())
    }

    fn local_access_service(&self) -> LocalAccessService {
        LocalAccessService::new(self.paths.clone())
    }

    pub fn list_codex_accounts(&self) -> Result<Vec<CodexAccount>, CodexError> {
        self.account_store().list_accounts()
    }

    pub fn get_current_codex_account(&self) -> Result<Option<CodexAccount>, CodexError> {
        self.account_store().get_current_account()
    }

    pub fn switch_codex_account(&self, account_id: String) -> Result<CodexAccount, CodexError> {
        self.account_store().switch_account_managed(&account_id)
    }

    pub fn delete_codex_account(&self, account_id: String) -> Result<(), CodexError> {
        self.account_store().delete_account(&account_id)
    }

    pub fn delete_codex_accounts(&self, account_ids: Vec<String>) -> Result<(), CodexError> {
        self.account_store().delete_multiple_accounts(&account_ids)
    }

    pub fn import_codex_from_local(&self) -> Result<Option<CodexAccount>, CodexError> {
        self.account_store().import_local_auth_json()
    }

    pub fn import_codex_from_json(
        &self,
        json_content: String,
    ) -> Result<Vec<CodexAccount>, CodexError> {
        Ok(self
            .account_store()
            .import_json_accounts(&json_content)?
            .imported)
    }

    pub fn import_codex_from_files(
        &self,
        file_paths: Vec<String>,
    ) -> Result<ImportResult, CodexError> {
        let store = self.account_store();
        let mut imported = Vec::new();
        let mut failed = Vec::new();

        for path in file_paths {
            match std::fs::read_to_string(&path)
                .map_err(CodexError::Io)
                .and_then(|content| store.import_json_accounts(&content))
            {
                Ok(result) => {
                    imported.extend(result.imported);
                    failed.extend(result.failed);
                }
                Err(err) => failed.push(crate::domain::account::ImportFailure {
                    account_id: path,
                    error: err.to_string(),
                }),
            }
        }

        Ok(ImportResult { imported, failed })
    }

    pub fn export_codex_accounts(&self, account_ids: Vec<String>) -> Result<String, CodexError> {
        let accounts = self.account_store().export_accounts(&account_ids)?;
        serde_json::to_string_pretty(&accounts).map_err(CodexError::Json)
    }

    pub fn update_codex_account_name(
        &self,
        account_id: String,
        name: String,
    ) -> Result<CodexAccount, CodexError> {
        let store = self.account_store();
        store.rename_account(&account_id, name)?;
        store.load_account_file(&account_id)
    }

    pub fn update_codex_account_tags(
        &self,
        account_id: String,
        tags: Vec<String>,
    ) -> Result<CodexAccount, CodexError> {
        let store = self.account_store();
        store.update_account_tags(&account_id, tags)?;
        store.load_account_file(&account_id)
    }

    pub fn add_codex_account_with_token(
        &self,
        id_token: String,
        access_token: String,
        refresh_token: Option<String>,
    ) -> Result<CodexAccount, CodexError> {
        let tokens = CodexTokens {
            access_token: Some(access_token),
            refresh_token,
            id_token: Some(id_token),
            token_type: Some("Bearer".into()),
            expires_at: None,
            scope: None,
        };
        let store = self.account_store();
        let account = store.account_from_oauth_tokens(&tokens)?;
        store.upsert_account(account.clone())?;
        Ok(account)
    }

    pub fn add_codex_account_with_api_key(
        &self,
        api_key: String,
        api_base_url: Option<String>,
        api_provider_mode: Option<CodexApiProviderMode>,
        api_provider_id: Option<String>,
        api_provider_name: Option<String>,
    ) -> Result<CodexAccount, CodexError> {
        self.account_store().upsert_api_key_account(
            &api_key,
            api_base_url.as_deref(),
            api_provider_mode,
            api_provider_id.as_deref(),
            api_provider_name.as_deref(),
        )
    }

    pub fn update_codex_api_key_credentials(
        &self,
        account_id: String,
        api_key: String,
        api_base_url: Option<String>,
        api_provider_mode: Option<CodexApiProviderMode>,
        api_provider_id: Option<String>,
        api_provider_name: Option<String>,
    ) -> Result<CodexAccount, CodexError> {
        self.account_store().update_api_key_credentials(
            &account_id,
            &api_key,
            api_base_url.as_deref(),
            api_provider_mode,
            api_provider_id.as_deref(),
            api_provider_name.as_deref(),
        )
    }

    pub fn codex_oauth_login_start(&self) -> Result<serde_json::Value, CodexError> {
        let service = OAuthService::new(self.paths.clone());
        let (auth_url, _, pending) = service.start_oauth_login(1455)?;
        Ok(serde_json::json!({
            "authUrl": auth_url,
            "auth_url": auth_url,
            "loginId": pending.login_id,
            "login_id": pending.login_id,
        }))
    }

    pub fn codex_oauth_submit_callback_url(
        &self,
        login_id: String,
        callback_url: String,
    ) -> Result<(), CodexError> {
        let service = OAuthService::new(self.paths.clone());
        let (code, state) = service.parse_manual_callback(&callback_url)?;
        let mut pending = service
            .load_pending()?
            .ok_or_else(|| CodexError::OAuth("No active OAuth login".into()))?;
        if pending.login_id != login_id {
            return Err(CodexError::AuthState(format!(
                "Stale login id: expected {} got {}",
                pending.login_id, login_id
            )));
        }
        service.complete_oauth_login(
            &[("code".into(), code.clone()), ("state".into(), state)],
            &pending,
        )?;
        pending.code = Some(code);
        service.save_pending(&pending)
    }

    pub async fn codex_oauth_login_completed(
        &self,
        login_id: String,
    ) -> Result<CodexAccount, CodexError> {
        let service = OAuthService::new(self.paths.clone());
        let pending = service
            .load_pending()?
            .ok_or_else(|| CodexError::OAuth("No active OAuth login".into()))?;
        let code = pending.code.clone().ok_or_else(|| {
            CodexError::OAuth("OAuth callback code has not been submitted".into())
        })?;
        service
            .complete_oauth_login_with_exchange(
                &login_id,
                &[("code".into(), code), ("state".into(), pending.state)],
            )
            .await
    }

    pub fn codex_oauth_login_cancel(&self, login_id: Option<String>) -> Result<(), CodexError> {
        let service = OAuthService::new(self.paths.clone());
        match login_id {
            Some(login_id) => service.cancel_login(&login_id),
            None => service.cancel_current(),
        }
    }

    pub fn get_codex_config_toml_path(&self) -> Result<String, CodexError> {
        Ok(self.paths.config_file.to_string_lossy().to_string())
    }

    pub fn get_general_config(&self) -> Result<serde_json::Value, CodexError> {
        serde_json::to_value(self.config_store().load_config()?).map_err(CodexError::Json)
    }

    pub fn save_general_config(&self, config: serde_json::Value) -> Result<(), CodexError> {
        let store = self.config_store();
        let mut current = store.load_config()?;
        let patch: UserConfigCodex = serde_json::from_value(config.clone())?;

        macro_rules! assign_some {
            ($field:ident) => {
                if patch.$field.is_some() {
                    current.$field = patch.$field;
                }
            };
        }

        assign_some!(codex_auto_refresh_minutes);
        assign_some!(codex_startup_wakeup_enabled);
        assign_some!(codex_startup_wakeup_delay_seconds);
        assign_some!(codex_app_path);
        assign_some!(codex_specified_app_path);
        assign_some!(codex_launch_on_switch);
        assign_some!(codex_restart_specified_app_on_switch);
        assign_some!(codex_local_access_entry_visible);
        assign_some!(codex_auto_switch_enabled);
        assign_some!(codex_auto_switch_primary_threshold);
        assign_some!(codex_auto_switch_secondary_threshold);
        assign_some!(codex_auto_switch_account_scope_mode);
        assign_some!(codex_auto_switch_selected_account_ids);
        assign_some!(codex_quota_alert_enabled);
        assign_some!(codex_quota_alert_threshold);
        assign_some!(codex_quota_alert_primary_threshold);
        assign_some!(codex_quota_alert_secondary_threshold);
        assign_some!(codex_quota_alert_cooldown_minutes);
        current.extra.extend(patch.extra);

        store.save_config(&current)
    }

    pub fn get_codex_quick_config(&self) -> Result<CodexQuickConfig, CodexError> {
        let config = self.config_store().load_config()?;
        Ok(CodexQuickConfig {
            model_context_window: None,
            model_auto_compact_token_limit: None,
        }
        .with_detected_defaults(config.codex_auto_refresh_minutes))
    }

    pub fn save_codex_quick_config(
        &self,
        model_context_window: Option<u64>,
        auto_compact_token_limit: Option<u64>,
    ) -> Result<CodexQuickConfig, CodexError> {
        Ok(CodexQuickConfig {
            model_context_window,
            model_auto_compact_token_limit: auto_compact_token_limit,
        })
    }

    pub fn codex_local_access_get_state(&self) -> Result<LocalAccessStateSnapshot, CodexError> {
        let service = self.local_access_service();
        let collection = service.load_collection()?;
        Ok(service.get_state_snapshot(&collection, false))
    }

    pub fn codex_local_access_save_accounts(
        &self,
        account_ids: Vec<String>,
        restrict_free_accounts: bool,
    ) -> Result<LocalAccessStateSnapshot, CodexError> {
        let service = self.local_access_service();
        let mut collection = service.load_collection()?;
        collection.restrict_free_accounts = restrict_free_accounts;
        service.save_accounts(&mut collection, &account_ids)?;
        Ok(service.get_state_snapshot(&collection, false))
    }

    pub fn codex_local_access_remove_account(
        &self,
        account_id: String,
    ) -> Result<LocalAccessStateSnapshot, CodexError> {
        let service = self.local_access_service();
        let mut collection = service.load_collection()?;
        collection.accounts.retain(|id| id != &account_id);
        service.save_collection(&collection)?;
        Ok(service.get_state_snapshot(&collection, false))
    }

    pub fn codex_local_access_rotate_api_key(
        &self,
    ) -> Result<LocalAccessStateSnapshot, CodexError> {
        let service = self.local_access_service();
        let mut collection = service.load_collection()?;
        service.rotate_api_key(&mut collection)?;
        Ok(service.get_state_snapshot(&collection, false))
    }

    pub fn codex_local_access_clear_stats(&self) -> Result<LocalAccessStateSnapshot, CodexError> {
        let service = self.local_access_service();
        service.clear_stats()?;
        let collection = service.load_collection()?;
        Ok(service.get_state_snapshot(&collection, false))
    }

    pub fn codex_local_access_prepare_restart(
        &self,
    ) -> Result<LocalAccessStateSnapshot, CodexError> {
        self.codex_local_access_get_state()
    }

    pub fn codex_local_access_update_port(
        &self,
        port: u16,
    ) -> Result<LocalAccessStateSnapshot, CodexError> {
        let service = self.local_access_service();
        let mut collection = service.load_collection()?;
        service.update_port(&mut collection, port)?;
        Ok(service.get_state_snapshot(&collection, false))
    }

    pub fn codex_local_access_update_routing_strategy(
        &self,
        strategy: RoutingStrategy,
    ) -> Result<LocalAccessStateSnapshot, CodexError> {
        let service = self.local_access_service();
        let mut collection = service.load_collection()?;
        service.update_routing(&mut collection, strategy)?;
        Ok(service.get_state_snapshot(&collection, false))
    }

    pub fn codex_local_access_set_enabled(
        &self,
        enabled: bool,
    ) -> Result<LocalAccessStateSnapshot, CodexError> {
        let service = self.local_access_service();
        let mut collection = service.load_collection()?;
        service.set_enabled(&mut collection, enabled)?;
        Ok(service.get_state_snapshot(&collection, false))
    }

    pub fn codex_local_access_activate(&self) -> Result<LocalAccessStateSnapshot, CodexError> {
        self.codex_local_access_set_enabled(true)
    }

    pub fn load_codex_account_groups(&self) -> Result<Vec<CodexAccountGroup>, CodexError> {
        let list: Option<CodexAccountGroupList> =
            read_json_file_opt(&self.paths.codex_account_groups_file)?;
        Ok(list.map(|list| list.groups).unwrap_or_default())
    }

    pub fn save_codex_account_groups(
        &self,
        groups: Vec<CodexAccountGroup>,
    ) -> Result<(), CodexError> {
        write_json_atomic(
            &self.paths.codex_account_groups_file,
            &CodexAccountGroupList { groups },
        )
    }

    pub fn create_codex_group(&self, name: String) -> Result<CodexAccountGroup, CodexError> {
        let mut store = GroupStore::from_groups(self.load_codex_account_groups()?);
        let id = store.create_group(name)?;
        let group = store
            .find_by_id(&id)
            .cloned()
            .ok_or_else(|| CodexError::NotFound(format!("Group not found: {id}")))?;
        self.save_codex_account_groups(store.groups().to_vec())?;
        Ok(group)
    }

    pub fn load_codex_model_providers(&self) -> Result<Vec<CodexModelProvider>, CodexError> {
        let list: Option<CodexModelProviderList> =
            read_json_file_opt(&self.paths.codex_model_providers_file)?;
        Ok(list.map(|list| list.providers).unwrap_or_default())
    }

    pub fn save_codex_model_providers(
        &self,
        providers: Vec<CodexModelProvider>,
    ) -> Result<(), CodexError> {
        let mut store = ModelProviderStore::from_providers(providers)?;
        store.cleanup_empty_preset_providers();
        write_json_atomic(&self.paths.codex_model_providers_file, &store.to_list())
    }
}

trait QuickConfigDefaults {
    fn with_detected_defaults(self, _refresh_minutes: Option<i32>) -> Self;
}

impl QuickConfigDefaults for CodexQuickConfig {
    fn with_detected_defaults(mut self, _refresh_minutes: Option<i32>) -> Self {
        self.model_context_window.get_or_insert(1_000_000);
        self.model_auto_compact_token_limit.get_or_insert(900_000);
        self
    }
}
