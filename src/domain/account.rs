use crate::adapters::fs_store::{
    read_json_file, read_json_file_opt, write_json_atomic, write_string_atomic, CodexPaths,
};
use crate::domain::api_key::validate_api_key;
use crate::domain::codex_models::*;
use crate::error::CodexError;
use serde_json::{Map, Value};
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex};
use toml_edit::value;

const DEFAULT_VERSION: &str = "1.0";
const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
static TOKEN_REFRESH_LOCKS: LazyLock<
    Mutex<std::collections::HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
> = LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

fn optional_non_empty(value: &Option<String>) -> Option<String> {
    value.as_deref().and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn required_token(value: &Option<String>, label: &str) -> Result<String, CodexError> {
    optional_non_empty(value).ok_or_else(|| {
        CodexError::AccountStore(format!(
            "OAuth account missing {label}, cannot write auth.json"
        ))
    })
}

fn normalize_non_default_base_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case(DEFAULT_OPENAI_BASE_URL) {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn derive_provider_id(name: &str, base_url: &str) -> String {
    let seed = format!("{name} {base_url}");
    let mut normalized = String::new();
    let mut previous_separator = false;
    for ch in seed.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            previous_separator = false;
        } else if !previous_separator {
            normalized.push('_');
            previous_separator = true;
        }
    }
    let normalized = normalized.trim_matches('_');
    if normalized.is_empty() {
        "custom".to_string()
    } else {
        normalized.to_string()
    }
}

pub struct AccountStore {
    pub paths: CodexPaths,
}

impl AccountStore {
    pub fn new(paths: CodexPaths) -> Self {
        Self { paths }
    }

    pub fn ensure_dirs(&self) -> Result<(), CodexError> {
        self.paths.ensure_dirs()
    }

    pub fn token_refresh_lock_for(account_id: &str) -> Arc<tokio::sync::Mutex<()>> {
        let mut locks = TOKEN_REFRESH_LOCKS
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        locks
            .entry(account_id.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone()
    }

    fn empty_index() -> CodexAccountIndex {
        CodexAccountIndex {
            version: DEFAULT_VERSION.to_string(),
            current_account_id: None,
            accounts: vec![],
        }
    }

    fn legacy_account_index_file(&self) -> std::path::PathBuf {
        self.paths.cockpit_dir.join("codex_account_index.json")
    }

    fn timestamp_to_epoch(value: Option<&str>) -> i64 {
        match value {
            Some(raw) => {
                if let Ok(n) = raw.parse::<i64>() {
                    if n > 10_000_000_000 {
                        n / 1000
                    } else {
                        n
                    }
                } else {
                    chrono::DateTime::parse_from_rfc3339(raw)
                        .map(|dt| dt.timestamp())
                        .unwrap_or(0)
                }
            }
            None => 0,
        }
    }

    fn summary_from_account(account: &CodexAccount) -> CodexAccountSummary {
        CodexAccountSummary {
            id: account.id.clone(),
            email: account.email.clone().unwrap_or_default(),
            plan_type: account.plan_type.clone(),
            subscription_active_until: None,
            created_at: Self::timestamp_to_epoch(account.created_at.as_deref()),
            last_used: Self::timestamp_to_epoch(account.last_used.as_deref()),
        }
    }

    fn normalize_account_for_storage(&self, account: &mut CodexAccount) {
        if account.provider.trim().is_empty() {
            account.provider = "codex".to_string();
        }

        if account.is_api_key() {
            if account
                .email
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                if let Some(api_key) = account.api_key.as_deref() {
                    account.email = Some(Self::build_api_key_email(api_key));
                }
            }
            account
                .plan_type
                .get_or_insert_with(|| "API_KEY".to_string());
            account
                .api_provider_mode
                .get_or_insert(CodexApiProviderMode::OpenAI);
            account.tokens.access_token.get_or_insert_with(String::new);
            account.tokens.id_token.get_or_insert_with(String::new);
        }

        if account.display_name.trim().is_empty() {
            account.display_name = account
                .email
                .clone()
                .or_else(|| account.provider_name.clone())
                .unwrap_or_else(|| account.id.clone());
        }

        let now = chrono::Utc::now().to_rfc3339();
        account.created_at.get_or_insert_with(|| now.clone());
        account.last_used.get_or_insert(now);
    }

    fn normalize_index(index: &mut CodexAccountIndex) {
        if index.version.trim().is_empty() {
            index.version = DEFAULT_VERSION.to_string();
        }
        if index.version == "1" {
            index.version = DEFAULT_VERSION.to_string();
        }
    }

    fn migrate_embedded_accounts_from_value(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), CodexError> {
        let Some(items) = value.get("accounts").and_then(|v| v.as_array()) else {
            return Ok(());
        };

        for item in items {
            let looks_like_full_account = item.get("auth_mode").is_some()
                || item.get("authMode").is_some()
                || item.get("tokens").is_some()
                || item.get("openai_api_key").is_some()
                || item.get("api_key").is_some();
            if !looks_like_full_account {
                continue;
            }
            if let Ok(mut account) = serde_json::from_value::<CodexAccount>(item.clone()) {
                if account.id.trim().is_empty() {
                    continue;
                }
                self.normalize_account_for_storage(&mut account);
                self.save_account_file(&account)?;
            }
        }
        Ok(())
    }

    fn load_index_from_path(&self, path: &Path) -> Result<CodexAccountIndex, CodexError> {
        let content = std::fs::read_to_string(path).map_err(CodexError::Io)?;
        if content.trim().is_empty() {
            return Ok(Self::empty_index());
        }
        let value: serde_json::Value = serde_json::from_str(&content)?;
        self.migrate_embedded_accounts_from_value(&value)?;
        let mut index: CodexAccountIndex = serde_json::from_value(value)?;
        Self::normalize_index(&mut index);
        Ok(index)
    }

    pub fn load_index(&self) -> Result<CodexAccountIndex, CodexError> {
        let primary = &self.paths.account_index_file;
        let legacy = self.legacy_account_index_file();

        let result = if primary.exists() {
            self.load_index_from_path(primary)
        } else if legacy.exists() {
            let index = self.load_index_from_path(&legacy)?;
            self.save_index(&index)?;
            Ok(index)
        } else {
            let empty = Self::empty_index();
            self.save_index(&empty)?;
            Ok(empty)
        };

        match result {
            Ok(index) => Ok(index),
            Err(CodexError::Json(_)) => {
                let empty = Self::empty_index();
                self.save_index(&empty)?;
                Ok(empty)
            }
            Err(e) => Err(e),
        }
    }

    pub fn save_index(&self, index: &CodexAccountIndex) -> Result<(), CodexError> {
        write_json_atomic(&self.paths.account_index_file, index)
    }

    pub fn save_account_file(&self, account: &CodexAccount) -> Result<(), CodexError> {
        let mut account = account.clone();
        self.normalize_account_for_storage(&mut account);
        write_json_atomic(&self.paths.account_file(&account.id), &account)
    }

    pub fn load_account_file(&self, account_id: &str) -> Result<CodexAccount, CodexError> {
        read_json_file(&self.paths.account_file(account_id))
    }

    pub fn load_account_file_opt(
        &self,
        account_id: &str,
    ) -> Result<Option<CodexAccount>, CodexError> {
        read_json_file_opt(&self.paths.account_file(account_id))
    }

    pub fn delete_account_file(&self, account_id: &str) -> Result<(), CodexError> {
        let path = self.paths.account_file(account_id);
        if path.exists() {
            std::fs::remove_file(&path).map_err(CodexError::Io)?;
        }
        Ok(())
    }

    pub fn get_current_account_id(&self) -> Result<Option<String>, CodexError> {
        let index = self.load_index()?;
        Ok(index.current_account_id)
    }

    pub fn get_current_account(&self) -> Result<Option<CodexAccount>, CodexError> {
        let index = self.load_index()?;
        match &index.current_account_id {
            Some(id) => self.load_account_file_opt(id),
            None => Ok(None),
        }
    }

    pub fn list_accounts(&self) -> Result<Vec<CodexAccount>, CodexError> {
        let index = self.load_index()?;
        let mut accounts = Vec::new();
        for summary in index.accounts {
            if let Some(account) = self.load_account_file_opt(&summary.id)? {
                accounts.push(account);
            }
        }
        Ok(accounts)
    }

    pub fn set_current_account(&self, account_id: &str) -> Result<(), CodexError> {
        let mut index = self.load_index()?;
        if !index.accounts.iter().any(|a| a.id == account_id) {
            return Err(CodexError::NotFound(format!(
                "Account not found: {}",
                account_id
            )));
        }
        index.current_account_id = Some(account_id.to_string());
        self.save_index(&index)
    }

    pub fn add_account(&self, mut account: CodexAccount) -> Result<(), CodexError> {
        let mut index = self.load_index()?;
        if index.accounts.iter().any(|a| a.id == account.id) {
            return Err(CodexError::AlreadyExists(format!(
                "Account already exists: {}",
                account.id
            )));
        }
        self.normalize_account_for_storage(&mut account);
        self.save_account_file(&account)?;
        index.accounts.push(Self::summary_from_account(&account));
        self.save_index(&index)
    }

    pub fn upsert_account(&self, mut account: CodexAccount) -> Result<(), CodexError> {
        let mut index = self.load_index()?;
        self.normalize_account_for_storage(&mut account);
        self.save_account_file(&account)?;
        let summary = Self::summary_from_account(&account);
        if let Some(pos) = index.accounts.iter().position(|a| a.id == account.id) {
            index.accounts[pos] = summary;
        } else {
            index.accounts.push(summary);
        }
        self.save_index(&index)
    }

    pub fn delete_account(&self, account_id: &str) -> Result<(), CodexError> {
        let mut index = self.load_index()?;
        let pos = index
            .accounts
            .iter()
            .position(|a| a.id == account_id)
            .ok_or_else(|| CodexError::NotFound(format!("Account not found: {}", account_id)))?;
        index.accounts.remove(pos);
        if index.current_account_id.as_deref() == Some(account_id) {
            index.current_account_id = None;
        }
        self.save_index(&index)?;
        self.delete_account_file(account_id)?;
        Ok(())
    }

    pub fn delete_multiple_accounts(&self, account_ids: &[String]) -> Result<(), CodexError> {
        let mut index = self.load_index()?;
        let ids_set: std::collections::HashSet<&str> =
            account_ids.iter().map(|s| s.as_str()).collect();
        index.accounts.retain(|a| !ids_set.contains(a.id.as_str()));
        if let Some(ref current) = index.current_account_id {
            if ids_set.contains(current.as_str()) {
                index.current_account_id = None;
            }
        }
        self.save_index(&index)?;
        for id in account_ids {
            let _ = self.delete_account_file(id);
        }
        Ok(())
    }

    fn update_index_summary_for_account(&self, account: &CodexAccount) -> Result<(), CodexError> {
        let mut index = self.load_index()?;
        let summary = Self::summary_from_account(account);
        if let Some(existing) = index.accounts.iter_mut().find(|a| a.id == account.id) {
            *existing = summary;
        } else {
            index.accounts.push(summary);
        }
        self.save_index(&index)
    }

    fn mutate_account<F>(&self, account_id: &str, mut f: F) -> Result<CodexAccount, CodexError>
    where
        F: FnMut(&mut CodexAccount),
    {
        let mut account = self.load_account_file(account_id)?;
        f(&mut account);
        self.normalize_account_for_storage(&mut account);
        self.save_account_file(&account)?;
        self.update_index_summary_for_account(&account)?;
        Ok(account)
    }

    pub fn update_account_tags(
        &self,
        account_id: &str,
        tags: Vec<String>,
    ) -> Result<(), CodexError> {
        let normalized: Vec<String> = tags
            .into_iter()
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();
        self.mutate_account(account_id, |account| {
            account.tags = normalized.clone();
        })?;
        Ok(())
    }

    pub fn rename_account(&self, account_id: &str, new_name: String) -> Result<(), CodexError> {
        self.mutate_account(account_id, |account| {
            account.display_name = new_name.clone();
        })?;
        Ok(())
    }

    pub fn update_account_quota(
        &self,
        account_id: &str,
        quota: CodexQuota,
    ) -> Result<(), CodexError> {
        self.mutate_account(account_id, |account| {
            account.quota = Some(Box::new(quota.clone()));
        })?;
        Ok(())
    }

    pub fn update_account_last_used(
        &self,
        account_id: &str,
        timestamp: &str,
    ) -> Result<(), CodexError> {
        let timestamp = timestamp.to_string();
        self.mutate_account(account_id, |account| {
            account.last_used = Some(timestamp.to_string());
        })?;
        Ok(())
    }

    pub fn update_account_tokens(
        &self,
        account_id: &str,
        tokens: CodexTokens,
    ) -> Result<(), CodexError> {
        self.mutate_account(account_id, |account| {
            account.tokens = tokens.clone();
        })?;
        Ok(())
    }

    pub fn write_auth_file(
        &self,
        path: &Path,
        auth_file: &CodexAuthFile,
    ) -> Result<(), CodexError> {
        write_json_atomic(path, auth_file)
    }

    pub fn load_auth_file(&self, path: &Path) -> Result<CodexAuthFile, CodexError> {
        read_json_file(path)
    }

    pub fn load_auth_file_opt(&self, path: &Path) -> Result<Option<CodexAuthFile>, CodexError> {
        read_json_file_opt(path)
    }

    pub fn import_local_auth_json(&self) -> Result<Option<CodexAccount>, CodexError> {
        let auth_file_path = &self.paths.auth_file;
        match self.load_auth_file_opt(auth_file_path)? {
            Some(auth_file) => {
                let account = match auth_file.auth_mode {
                    CodexAuthMode::OAuth => {
                        let tokens = auth_file.tokens.unwrap_or_default();
                        self.account_from_oauth_tokens(&tokens)?
                    }
                    CodexAuthMode::ApiKey => {
                        let api_key = auth_file.api_key.clone().ok_or_else(|| {
                            CodexError::Import("auth.json missing OPENAI_API_KEY".into())
                        })?;
                        self.upsert_api_key_account(
                            &api_key,
                            auth_file.base_url.as_deref(),
                            None,
                            None,
                            None,
                        )?
                    }
                };
                if account.is_oauth() {
                    self.upsert_account(account.clone())?;
                }
                Ok(Some(account))
            }
            None => Ok(None),
        }
    }

    pub fn import_json_accounts(&self, json_data: &str) -> Result<ImportResult, CodexError> {
        let mut imported = Vec::new();
        let mut failed = Vec::new();

        let value: serde_json::Value = serde_json::from_str(json_data)
            .map_err(|e| CodexError::Import(format!("Invalid import JSON: {e}")))?;

        let candidates = match value {
            serde_json::Value::Array(items) => items,
            serde_json::Value::Object(ref object) => {
                if let Some(accounts) = object.get("accounts").and_then(|v| v.as_array()) {
                    accounts.clone()
                } else {
                    vec![serde_json::Value::Object(object.clone())]
                }
            }
            _ => return Err(CodexError::Import("Invalid import format".into())),
        };

        for candidate in candidates {
            let label = candidate
                .get("id")
                .and_then(|v| v.as_str())
                .or_else(|| candidate.get("email").and_then(|v| v.as_str()))
                .unwrap_or("unknown")
                .to_string();

            match self.import_json_candidate(candidate) {
                Ok(Some(account)) => imported.push(account),
                Ok(None) => failed.push(ImportFailure {
                    account_id: label,
                    error: "Unsupported account import object".into(),
                }),
                Err(e) => failed.push(ImportFailure {
                    account_id: label,
                    error: e.to_string(),
                }),
            }
        }

        if imported.is_empty() && !failed.is_empty() {
            return Err(CodexError::Import(format!(
                "No accounts imported: {}",
                failed
                    .iter()
                    .map(|f| format!("{}: {}", f.account_id, f.error))
                    .collect::<Vec<_>>()
                    .join("; ")
            )));
        }

        Ok(ImportResult { imported, failed })
    }

    fn import_json_candidate(
        &self,
        candidate: serde_json::Value,
    ) -> Result<Option<CodexAccount>, CodexError> {
        if let Some(api_key) = Self::extract_api_key_from_value(&candidate) {
            let base_url = Self::extract_base_url_from_value(&candidate);
            let provider_id = candidate
                .get("api_provider_id")
                .or_else(|| candidate.get("provider_id"))
                .and_then(|v| v.as_str());
            let provider_name = candidate
                .get("api_provider_name")
                .or_else(|| candidate.get("provider_name"))
                .and_then(|v| v.as_str());
            return self
                .upsert_api_key_account(
                    &api_key,
                    base_url.as_deref(),
                    None,
                    provider_id,
                    provider_name,
                )
                .map(Some);
        }

        if let Ok(account) = serde_json::from_value::<CodexAccount>(candidate.clone()) {
            return self.import_single_account(account).map(Some);
        }

        if let Ok(auth_file) = serde_json::from_value::<CodexAuthFile>(candidate) {
            if let Some(tokens) = auth_file.tokens {
                if tokens.id_token.is_some() || tokens.access_token.is_some() {
                    let account = self.account_from_oauth_tokens(&tokens)?;
                    self.upsert_account(account.clone())?;
                    return Ok(Some(account));
                }
            }
        }

        Ok(None)
    }

    fn extract_api_key_from_value(value: &serde_json::Value) -> Option<String> {
        [
            "OPENAI_API_KEY",
            "openai_api_key",
            "api_key",
            "apiKey",
            "key",
        ]
        .iter()
        .find_map(|key| {
            value
                .get(*key)
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(ToString::to_string)
        })
    }

    fn extract_base_url_from_value(value: &serde_json::Value) -> Option<String> {
        ["api_base_url", "apiBaseUrl", "base_url", "baseUrl"]
            .iter()
            .find_map(|key| {
                value
                    .get(*key)
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(ToString::to_string)
            })
    }

    fn import_single_account(&self, account: CodexAccount) -> Result<CodexAccount, CodexError> {
        if account.id.is_empty() {
            return Err(CodexError::Import("Account ID is required".into()));
        }
        self.upsert_account(account.clone())?;
        Ok(account)
    }

    pub fn upsert_api_key_account(
        &self,
        api_key: &str,
        api_base_url: Option<&str>,
        api_provider_mode: Option<CodexApiProviderMode>,
        api_provider_id: Option<&str>,
        api_provider_name: Option<&str>,
    ) -> Result<CodexAccount, CodexError> {
        let validation =
            validate_api_key(api_key, api_base_url.unwrap_or_default(), api_provider_name)?;
        if !validation.is_valid {
            return Err(CodexError::ApiKey(
                validation
                    .error
                    .unwrap_or_else(|| "Invalid API key credentials".into()),
            ));
        }

        let normalized_key = api_key.trim().to_string();
        let normalized_base_url = validation.normalized_base_url;
        let is_openai_default = normalized_base_url == DEFAULT_OPENAI_BASE_URL;
        let provider_mode = api_provider_mode.unwrap_or(if is_openai_default {
            CodexApiProviderMode::OpenAI
        } else {
            CodexApiProviderMode::Custom
        });
        let account_id = Self::build_api_key_account_id(&normalized_key);
        let now = chrono::Utc::now().to_rfc3339();

        let mut account = self
            .load_account_file_opt(&account_id)?
            .unwrap_or(CodexAccount {
                id: account_id.clone(),
                provider: "codex".into(),
                auth_mode: CodexAuthMode::ApiKey,
                email: Some(Self::build_api_key_email(&normalized_key)),
                plan_type: Some("API_KEY".into()),
                account_id: None,
                organization_id: None,
                organizations: vec![],
                display_name: api_provider_name
                    .filter(|name| !name.trim().is_empty())
                    .unwrap_or("OpenAI API Key")
                    .to_string(),
                tags: vec!["api-key".into()],
                tokens: CodexTokens {
                    access_token: Some(String::new()),
                    refresh_token: None,
                    id_token: Some(String::new()),
                    token_type: None,
                    expires_at: None,
                    scope: None,
                },
                api_key: Some(normalized_key.clone()),
                base_url: None,
                provider_id: None,
                provider_name: None,
                api_provider_mode: Some(provider_mode.clone()),
                quota: None,
                created_at: Some(now.clone()),
                last_used: Some(now.clone()),
                last_refresh: None,
            });

        account.auth_mode = CodexAuthMode::ApiKey;
        account.email = Some(Self::build_api_key_email(&normalized_key));
        account.plan_type = Some("API_KEY".into());
        account.api_key = Some(normalized_key);
        account.base_url = if is_openai_default {
            None
        } else {
            Some(normalized_base_url)
        };
        account.provider_id = api_provider_id
            .map(ToString::to_string)
            .or(Some(validation.provider_id));
        account.provider_name = api_provider_name.map(ToString::to_string).or_else(|| {
            if is_openai_default {
                Some("OpenAI".to_string())
            } else {
                None
            }
        });
        account.api_provider_mode = Some(provider_mode);
        account.last_used = Some(now);

        self.upsert_account(account.clone())?;
        Ok(account)
    }

    pub fn update_api_key_credentials(
        &self,
        account_id: &str,
        api_key: &str,
        api_base_url: Option<&str>,
        api_provider_mode: Option<CodexApiProviderMode>,
        api_provider_id: Option<&str>,
        api_provider_name: Option<&str>,
    ) -> Result<CodexAccount, CodexError> {
        let existing = self.load_account_file(account_id)?;
        if !existing.is_api_key() {
            return Err(CodexError::InvalidState(
                "Only API key accounts support credential editing".into(),
            ));
        }

        let updated = self.upsert_api_key_account(
            api_key,
            api_base_url,
            api_provider_mode,
            api_provider_id,
            api_provider_name,
        )?;

        if updated.id != existing.id {
            self.delete_account_file(&existing.id)?;
            let mut index = self.load_index()?;
            if index.current_account_id.as_deref() == Some(existing.id.as_str()) {
                index.current_account_id = Some(updated.id.clone());
            }
            index
                .accounts
                .retain(|item| item.id != existing.id && item.id != updated.id);
            index.accounts.push(Self::summary_from_account(&updated));
            self.save_index(&index)?;
        }

        if self.get_current_account_id()?.as_deref() == Some(updated.id.as_str()) {
            let auth = self.build_auth_file_for_account(&updated)?;
            self.write_auth_file(&self.paths.auth_file, &auth)?;
        }

        Ok(updated)
    }

    fn build_api_key_account_id(api_key: &str) -> String {
        format!("codex_apikey_{:x}", md5::compute(api_key.as_bytes()))
    }

    fn build_api_key_email(api_key: &str) -> String {
        let hash = format!("{:x}", md5::compute(api_key.as_bytes()));
        format!("api-key-{}", &hash[..8])
    }

    pub fn export_accounts(&self, account_ids: &[String]) -> Result<Vec<CodexAccount>, CodexError> {
        let ids = if account_ids.is_empty() {
            let index = self.load_index()?;
            index.accounts.iter().map(|a| a.id.clone()).collect()
        } else {
            account_ids.to_vec()
        };

        let mut result = Vec::new();
        for id in &ids {
            if let Some(account) = self.load_account_file_opt(id)? {
                let sanitized = self.sanitize_for_export(account);
                result.push(sanitized);
            }
        }
        Ok(result)
    }

    fn sanitize_for_export(&self, account: CodexAccount) -> CodexAccount {
        account
    }

    pub fn account_from_oauth_tokens(
        &self,
        tokens: &CodexTokens,
    ) -> Result<CodexAccount, CodexError> {
        if let Some(ref id_token) = tokens.id_token {
            use crate::domain::oauth::{decode_jwt_payload, extract_account_from_tokens};
            let (payload, _) = decode_jwt_payload(id_token)?;
            extract_account_from_tokens(tokens, Some(&payload))
        } else {
            let id = format!(
                "acct_oauth_{}",
                &uuid::Uuid::new_v4().to_string().replace('-', "_")[..16]
            );
            Ok(CodexAccount {
                id,
                provider: "codex".into(),
                auth_mode: CodexAuthMode::OAuth,
                email: None,
                plan_type: None,
                account_id: None,
                organization_id: None,
                organizations: vec![],
                display_name: "OAuth Account".into(),
                tags: vec![],
                tokens: tokens.clone(),
                api_key: None,
                base_url: None,
                provider_id: None,
                provider_name: None,
                api_provider_mode: None,
                quota: None,
                created_at: Some(chrono::Utc::now().to_rfc3339()),
                last_used: None,
                last_refresh: Some(chrono::Utc::now().to_rfc3339()),
            })
        }
    }

    pub fn profile_refresh(
        &self,
        account_id: &str,
        profile_data: &AccountProfile,
    ) -> Result<(), CodexError> {
        self.mutate_account(account_id, |account| {
            if let Some(ref email) = profile_data.email {
                account.email = Some(email.clone());
            }
            if let Some(ref plan_type) = profile_data.plan_type {
                account.plan_type = Some(plan_type.clone());
            }
            if !profile_data.organizations.is_empty() {
                account.organizations = profile_data.organizations.clone();
            }
            account.last_refresh = Some(chrono::Utc::now().to_rfc3339());
        })?;
        Ok(())
    }

    pub fn switch_account_managed(&self, account_id: &str) -> Result<CodexAccount, CodexError> {
        let account = self.load_account_file(account_id)?;
        let now = chrono::Utc::now().to_rfc3339();

        let auth_file = self.build_official_auth_file_value(&account, &now)?;
        write_json_atomic(&self.paths.auth_file, &auth_file)?;
        self.write_api_provider_to_config_toml(&account)?;

        self.set_current_account(account_id)?;

        self.update_account_last_used(account_id, &now)?;

        Ok(account)
    }

    fn build_official_auth_file_value(
        &self,
        account: &CodexAccount,
        now: &str,
    ) -> Result<Value, CodexError> {
        match account.auth_mode {
            CodexAuthMode::OAuth => {
                let id_token = required_token(&account.tokens.id_token, "id_token")?;
                let access_token = required_token(&account.tokens.access_token, "access_token")?;

                let mut tokens = Map::new();
                tokens.insert("id_token".into(), Value::String(id_token));
                tokens.insert("access_token".into(), Value::String(access_token));
                if let Some(refresh_token) = optional_non_empty(&account.tokens.refresh_token) {
                    tokens.insert("refresh_token".into(), Value::String(refresh_token));
                }
                if let Some(account_id) = optional_non_empty(&account.account_id) {
                    tokens.insert("account_id".into(), Value::String(account_id));
                }

                let mut root = Map::new();
                root.insert("OPENAI_API_KEY".into(), Value::Null);
                root.insert("tokens".into(), Value::Object(tokens));
                root.insert("last_refresh".into(), Value::String(now.to_string()));
                Ok(Value::Object(root))
            }
            CodexAuthMode::ApiKey => {
                let api_key = optional_non_empty(&account.api_key).ok_or_else(|| {
                    CodexError::AccountStore("API key account missing credentials".into())
                })?;
                let mut root = Map::new();
                root.insert("auth_mode".into(), Value::String("apikey".into()));
                root.insert("OPENAI_API_KEY".into(), Value::String(api_key));
                Ok(Value::Object(root))
            }
        }
    }

    fn write_api_provider_to_config_toml(&self, account: &CodexAccount) -> Result<(), CodexError> {
        let provider_mode = if account.auth_mode == CodexAuthMode::ApiKey {
            account
                .api_provider_mode
                .clone()
                .unwrap_or(CodexApiProviderMode::OpenAI)
        } else {
            CodexApiProviderMode::OpenAI
        };
        let base_url = if account.auth_mode == CodexAuthMode::ApiKey {
            account
                .base_url
                .as_deref()
                .and_then(normalize_non_default_base_url)
        } else {
            None
        };

        if !self.paths.config_file.exists() && base_url.is_none() {
            return Ok(());
        }

        let existing = std::fs::read_to_string(&self.paths.config_file).unwrap_or_default();
        let mut doc = if existing.trim().is_empty() {
            toml_edit::DocumentMut::new()
        } else {
            existing
                .parse::<toml_edit::DocumentMut>()
                .map_err(|e| CodexError::Toml(format!("Failed to parse config TOML: {e}")))?
        };

        match provider_mode {
            CodexApiProviderMode::OpenAI => {
                doc.remove("model_provider");
                match base_url {
                    Some(base_url) => {
                        doc["openai_base_url"] = value(base_url.as_str());
                    }
                    None => {
                        doc.remove("openai_base_url");
                    }
                }
            }
            CodexApiProviderMode::Custom | CodexApiProviderMode::Azure => {
                doc.remove("openai_base_url");
                let base_url = base_url.ok_or_else(|| {
                    CodexError::Config("Custom Codex API provider missing base URL".into())
                })?;
                let provider_id = optional_non_empty(&account.provider_id).unwrap_or_else(|| {
                    derive_provider_id(
                        account.provider_name.as_deref().unwrap_or("custom"),
                        base_url.as_str(),
                    )
                });
                let provider_name = optional_non_empty(&account.provider_name)
                    .unwrap_or_else(|| provider_id.clone());

                doc["model_provider"] = value(provider_id.as_str());
                if !doc.contains_key("model_providers") {
                    doc["model_providers"] = toml_edit::table();
                }
                let model_providers = doc["model_providers"].as_table_mut().ok_or_else(|| {
                    CodexError::Config("config.toml model_providers is not a table".into())
                })?;
                if !model_providers.contains_key(&provider_id) {
                    model_providers[provider_id.as_str()] = toml_edit::table();
                }
                let provider_table = model_providers[provider_id.as_str()]
                    .as_table_mut()
                    .ok_or_else(|| {
                        CodexError::Config(
                            "config.toml target model provider is not a table".into(),
                        )
                    })?;
                provider_table["name"] = value(provider_name.as_str());
                provider_table["base_url"] = value(base_url.as_str());
                provider_table["wire_api"] = value("responses");
                provider_table["requires_openai_auth"] = value(true);
            }
        }

        write_string_atomic(&self.paths.config_file, &doc.to_string())
    }

    pub fn build_auth_file_for_account(
        &self,
        account: &CodexAccount,
    ) -> Result<CodexAuthFile, CodexError> {
        match account.auth_mode {
            CodexAuthMode::OAuth => Ok(CodexAuthFile {
                auth_mode: CodexAuthMode::OAuth,
                tokens: Some(account.tokens.clone()),
                api_key: None,
                base_url: None,
            }),
            CodexAuthMode::ApiKey => {
                if account.api_key.is_none() {
                    return Err(CodexError::AccountStore(
                        "API key account missing credentials".into(),
                    ));
                }
                Ok(CodexAuthFile {
                    auth_mode: CodexAuthMode::ApiKey,
                    tokens: None,
                    api_key: account.api_key.clone(),
                    base_url: account.base_url.clone(),
                })
            }
        }
    }

    pub fn prepare_account_for_injection(
        &self,
        account_id: &str,
    ) -> Result<CodexAuthFile, CodexError> {
        let account = self.load_account_file(account_id)?;
        self.build_auth_file_for_account(&account)
    }

    pub fn activate_api_key_for_local_access(
        &self,
        api_key: &str,
        base_url: &str,
    ) -> Result<(), CodexError> {
        let auth = CodexAuthFile {
            auth_mode: CodexAuthMode::ApiKey,
            tokens: None,
            api_key: Some(api_key.to_string()),
            base_url: Some(base_url.to_string()),
        };
        self.write_auth_file(&self.paths.auth_file, &auth)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportResult {
    pub imported: Vec<CodexAccount>,
    pub failed: Vec<ImportFailure>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportFailure {
    pub account_id: String,
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_store() -> (AccountStore, TempDir) {
        let tmp = TempDir::new().expect("temp dir");
        let paths = CodexPaths::for_tests(tmp.path());
        let store = AccountStore::new(paths);
        store.ensure_dirs().expect("ensure dirs");
        (store, tmp)
    }

    fn sample_oauth_account(id: &str, email: &str) -> CodexAccount {
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
                access_token: Some(format!("at_{id}")),
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
    fn test_empty_store_returns_empty_index() {
        let (store, _tmp) = setup_test_store();
        let accounts = store.list_accounts().expect("list");
        assert!(accounts.is_empty());
    }

    #[test]
    fn test_add_and_list_accounts() {
        let (store, _tmp) = setup_test_store();
        let account = sample_oauth_account("test_001", "test@example.com");
        store.add_account(account.clone()).expect("add");
        let accounts = store.list_accounts().expect("list");
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].id, "test_001");
    }

    #[test]
    fn test_duplicate_account_rejected() {
        let (store, _tmp) = setup_test_store();
        let account = sample_oauth_account("test_001", "test@example.com");
        store.add_account(account.clone()).expect("add");
        let result = store.add_account(account);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CodexError::AlreadyExists(_)));
    }

    #[test]
    fn test_set_and_get_current_account() {
        let (store, _tmp) = setup_test_store();
        let account = sample_oauth_account("test_001", "test@example.com");
        store.add_account(account).expect("add");
        store.set_current_account("test_001").expect("set current");
        let current = store.get_current_account().expect("get current");
        assert!(current.is_some());
        assert_eq!(current.unwrap().id, "test_001");
    }

    #[test]
    fn test_set_current_nonexistent_fails() {
        let (store, _tmp) = setup_test_store();
        let result = store.set_current_account("no_exist");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_account() {
        let (store, _tmp) = setup_test_store();
        let account = sample_oauth_account("test_001", "test@example.com");
        store.add_account(account).expect("add");
        store.set_current_account("test_001").expect("set current");
        store.delete_account("test_001").expect("delete");
        let accounts = store.list_accounts().expect("list");
        assert!(accounts.is_empty());
        let current = store.get_current_account_id().expect("current id");
        assert!(current.is_none());
    }

    #[test]
    fn test_delete_multiple_accounts() {
        let (store, _tmp) = setup_test_store();
        store
            .add_account(sample_oauth_account("test_001", "a@example.com"))
            .expect("add1");
        store
            .add_account(sample_oauth_account("test_002", "b@example.com"))
            .expect("add2");
        store
            .add_account(sample_oauth_account("test_003", "c@example.com"))
            .expect("add3");
        store.set_current_account("test_001").expect("set current");
        store
            .delete_multiple_accounts(&["test_001".into(), "test_002".into()])
            .expect("delete multi");
        let accounts = store.list_accounts().expect("list");
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].id, "test_003");
        let current = store.get_current_account_id().expect("current id");
        assert!(current.is_none());
    }

    #[test]
    fn test_upsert_account() {
        let (store, _tmp) = setup_test_store();
        let mut account = sample_oauth_account("test_001", "old@example.com");
        store.upsert_account(account.clone()).expect("upsert1");
        account.display_name = "Updated Name".into();
        store.upsert_account(account.clone()).expect("upsert2");
        let loaded = store.load_account_file("test_001").expect("load");
        assert_eq!(loaded.display_name, "Updated Name");
    }

    #[test]
    fn test_token_refresh_lock_is_per_account() {
        let a1 = AccountStore::token_refresh_lock_for("acct_a");
        let a2 = AccountStore::token_refresh_lock_for("acct_a");
        let b = AccountStore::token_refresh_lock_for("acct_b");

        assert!(Arc::ptr_eq(&a1, &a2));
        assert!(!Arc::ptr_eq(&a1, &b));
    }

    #[test]
    fn test_update_tags() {
        let (store, _tmp) = setup_test_store();
        store
            .add_account(sample_oauth_account("test_001", "test@example.com"))
            .expect("add");
        store
            .update_account_tags("test_001", vec!["New".into(), "  TAG  ".into(), "".into()])
            .expect("update tags");
        let loaded = store.load_account_file("test_001").expect("load");
        assert_eq!(loaded.tags, vec!["new", "tag"]);
    }

    #[test]
    fn test_rename_account() {
        let (store, _tmp) = setup_test_store();
        store
            .add_account(sample_oauth_account("test_001", "test@example.com"))
            .expect("add");
        store
            .rename_account("test_001", "New Display Name".into())
            .expect("rename");
        let loaded = store.load_account_file("test_001").expect("load");
        assert_eq!(loaded.display_name, "New Display Name");
    }

    #[test]
    fn test_quota_update() {
        let (store, _tmp) = setup_test_store();
        store
            .add_account(sample_oauth_account("test_001", "test@example.com"))
            .expect("add");
        let quota = CodexQuota {
            account_id: Some("acct_test_001".into()),
            plan_type: Some("pro".into()),
            windows: vec![],
            code_review_quota: None,
            error: None,
            retry_after_ms: None,
            raw_data: None,
            ..CodexQuota::default()
        };
        store
            .update_account_quota("test_001", quota)
            .expect("update quota");
        let loaded = store.load_account_file("test_001").expect("load");
        assert!(loaded.quota.is_some());
    }

    #[test]
    fn test_load_missing_account_returns_none_opt() {
        let (store, _tmp) = setup_test_store();
        let result = store.load_account_file_opt("no_exist").expect("load opt");
        assert!(result.is_none());
    }

    #[test]
    fn test_load_missing_account_returns_err() {
        let (store, _tmp) = setup_test_store();
        let result = store.load_account_file("no_exist");
        assert!(result.is_err());
    }

    #[test]
    fn test_write_and_load_auth_file() {
        let (store, tmp) = setup_test_store();
        let auth = CodexAuthFile {
            auth_mode: CodexAuthMode::OAuth,
            tokens: Some(CodexTokens {
                access_token: Some("at_test".into()),
                refresh_token: Some("rt_test".into()),
                id_token: Some("jwt_test".into()),
                token_type: Some("Bearer".into()),
                expires_at: Some("2026-05-04T00:00:00Z".into()),
                scope: None,
            }),
            api_key: None,
            base_url: None,
        };
        let path = tmp.path().join("test_auth.json");
        store.write_auth_file(&path, &auth).expect("write");
        let loaded = store.load_auth_file(&path).expect("load");
        assert_eq!(loaded.auth_mode, CodexAuthMode::OAuth);
        assert!(loaded.tokens.is_some());
    }

    #[test]
    fn test_corrupt_index_recovers() {
        let (store, _tmp) = setup_test_store();
        std::fs::write(&store.paths.account_index_file, "this is not json").expect("write");
        let index = store.load_index().expect("load index");
        assert!(index.accounts.is_empty());
    }

    #[test]
    fn test_empty_index_file() {
        let (store, _tmp) = setup_test_store();
        std::fs::write(&store.paths.account_index_file, "").expect("write");
        let result = read_json_file_opt::<CodexAccountIndex>(&store.paths.account_index_file);
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn test_apikey_account_no_tokens() {
        let (store, _tmp) = setup_test_store();
        let account = sample_apikey_account("apikey_001");
        store.add_account(account).expect("add");
        let loaded = store.load_account_file("apikey_001").expect("load");
        assert!(loaded.is_api_key());
        assert_eq!(loaded.tokens.access_token, Some(String::new()));
        assert!(loaded.api_key.is_some());
    }
}
