use crate::adapters::fs_store::{
    read_json_file, read_json_file_opt, write_json_atomic, CodexPaths,
};
use crate::domain::codex_models::*;
use crate::error::CodexError;
use std::path::Path;

const DEFAULT_VERSION: u32 = 1;

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

    pub fn load_index(&self) -> Result<CodexAccountIndex, CodexError> {
        match read_json_file_opt(&self.paths.account_index_file) {
            Ok(Some(index)) => Ok(index),
            Ok(None) => {
                let empty = CodexAccountIndex {
                    version: DEFAULT_VERSION,
                    current_account_id: None,
                    accounts: vec![],
                };
                write_json_atomic(&self.paths.account_index_file, &empty)?;
                Ok(empty)
            }
            Err(e) => {
                if let CodexError::Json(_) = &e {
                    let empty = CodexAccountIndex {
                        version: DEFAULT_VERSION,
                        current_account_id: None,
                        accounts: vec![],
                    };
                    write_json_atomic(&self.paths.account_index_file, &empty)?;
                    Ok(empty)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn save_index(&self, index: &CodexAccountIndex) -> Result<(), CodexError> {
        write_json_atomic(&self.paths.account_index_file, index)
    }

    pub fn save_account_file(&self, account: &CodexAccount) -> Result<(), CodexError> {
        write_json_atomic(&self.paths.account_file(&account.id), account)
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
        Ok(index.accounts)
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

    pub fn add_account(&self, account: CodexAccount) -> Result<(), CodexError> {
        let mut index = self.load_index()?;
        if index.accounts.iter().any(|a| a.id == account.id) {
            return Err(CodexError::AlreadyExists(format!(
                "Account already exists: {}",
                account.id
            )));
        }
        self.save_account_file(&account)?;
        index.accounts.push(account);
        self.save_index(&index)
    }

    pub fn upsert_account(&self, account: CodexAccount) -> Result<(), CodexError> {
        let mut index = self.load_index()?;
        self.save_account_file(&account)?;
        if let Some(pos) = index.accounts.iter().position(|a| a.id == account.id) {
            index.accounts[pos] = account;
        } else {
            index.accounts.push(account);
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

    pub fn update_account_tags(
        &self,
        account_id: &str,
        tags: Vec<String>,
    ) -> Result<(), CodexError> {
        let mut index = self.load_index()?;
        {
            let account = index
                .accounts
                .iter_mut()
                .find(|a| a.id == account_id)
                .ok_or_else(|| {
                    CodexError::NotFound(format!("Account not found: {}", account_id))
                })?;
            let normalized: Vec<String> = tags
                .into_iter()
                .map(|t| t.trim().to_lowercase())
                .filter(|t| !t.is_empty())
                .collect();
            account.tags = normalized;
        }
        let account_ref = index
            .accounts
            .iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| CodexError::NotFound(format!("Account not found: {}", account_id)))?;
        let account_clone = account_ref.clone();
        self.save_index(&index)?;
        self.save_account_file(&account_clone)?;
        Ok(())
    }

    pub fn rename_account(&self, account_id: &str, new_name: String) -> Result<(), CodexError> {
        let mut index = self.load_index()?;
        {
            let account = index
                .accounts
                .iter_mut()
                .find(|a| a.id == account_id)
                .ok_or_else(|| {
                    CodexError::NotFound(format!("Account not found: {}", account_id))
                })?;
            account.display_name = new_name;
        }
        let account_ref = index
            .accounts
            .iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| CodexError::NotFound(format!("Account not found: {}", account_id)))?;
        let account_clone = account_ref.clone();
        self.save_index(&index)?;
        self.save_account_file(&account_clone)?;
        Ok(())
    }

    pub fn update_account_quota(
        &self,
        account_id: &str,
        quota: CodexQuota,
    ) -> Result<(), CodexError> {
        let mut index = self.load_index()?;
        {
            let account = index
                .accounts
                .iter_mut()
                .find(|a| a.id == account_id)
                .ok_or_else(|| {
                    CodexError::NotFound(format!("Account not found: {}", account_id))
                })?;
            account.quota = Some(Box::new(quota));
        }
        let account_ref = index
            .accounts
            .iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| CodexError::NotFound(format!("Account not found: {}", account_id)))?;
        let account_clone = account_ref.clone();
        self.save_index(&index)?;
        self.save_account_file(&account_clone)?;
        Ok(())
    }

    pub fn update_account_last_used(
        &self,
        account_id: &str,
        timestamp: &str,
    ) -> Result<(), CodexError> {
        let mut index = self.load_index()?;
        {
            let account = index
                .accounts
                .iter_mut()
                .find(|a| a.id == account_id)
                .ok_or_else(|| {
                    CodexError::NotFound(format!("Account not found: {}", account_id))
                })?;
            account.last_used = Some(timestamp.to_string());
        }
        let account_ref = index
            .accounts
            .iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| CodexError::NotFound(format!("Account not found: {}", account_id)))?;
        let account_clone = account_ref.clone();
        self.save_index(&index)?;
        self.save_account_file(&account_clone)?;
        Ok(())
    }

    pub fn update_account_tokens(
        &self,
        account_id: &str,
        tokens: CodexTokens,
    ) -> Result<(), CodexError> {
        let mut index = self.load_index()?;
        {
            let account = index
                .accounts
                .iter_mut()
                .find(|a| a.id == account_id)
                .ok_or_else(|| {
                    CodexError::NotFound(format!("Account not found: {}", account_id))
                })?;
            account.tokens = tokens;
        }
        let account_ref = index
            .accounts
            .iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| CodexError::NotFound(format!("Account not found: {}", account_id)))?;
        let account_clone = account_ref.clone();
        self.save_index(&index)?;
        self.save_account_file(&account_clone)?;
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
        assert_eq!(loaded.tokens.access_token, None);
        assert!(loaded.api_key.is_some());
    }
}
