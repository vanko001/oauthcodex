use crate::domain::api_key::generate_api_key_id;
use crate::domain::codex_models::{CodexModelApiKey, CodexModelProvider, CodexModelProviderList};
use crate::error::CodexError;
use std::collections::HashMap;

pub struct ModelProviderStore {
    providers: Vec<CodexModelProvider>,
    provider_index: HashMap<String, usize>,
    api_key_index: HashMap<String, String>,
}

impl ModelProviderStore {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            provider_index: HashMap::new(),
            api_key_index: HashMap::new(),
        }
    }

    pub fn from_providers(providers: Vec<CodexModelProvider>) -> Result<Self, CodexError> {
        let mut store = Self::new();
        for p in providers {
            store.add_provider(p)?;
        }
        Ok(store)
    }

    pub fn providers(&self) -> &[CodexModelProvider] {
        &self.providers
    }

    pub fn to_list(&self) -> CodexModelProviderList {
        CodexModelProviderList {
            providers: self.providers.clone(),
        }
    }

    pub fn find_by_base_url(&self, base_url: &str) -> Option<&CodexModelProvider> {
        self.providers.iter().find(|p| p.base_url == base_url)
    }

    pub fn find_by_id(&self, id: &str) -> Option<&CodexModelProvider> {
        self.provider_index.get(id).map(|&idx| &self.providers[idx])
    }

    pub fn add_provider(&mut self, provider: CodexModelProvider) -> Result<(), CodexError> {
        if self.find_by_id(&provider.id).is_some() {
            return Err(CodexError::AlreadyExists(format!(
                "Provider {} already exists",
                provider.id
            )));
        }

        if self.find_by_base_url(&provider.base_url).is_some() {
            return Err(CodexError::AlreadyExists(format!(
                "Provider with base URL {} already exists",
                provider.base_url
            )));
        }

        if provider.name.trim().is_empty() {
            return Err(CodexError::Provider("Provider name cannot be empty".into()));
        }

        let idx = self.providers.len();
        self.provider_index.insert(provider.id.clone(), idx);
        for key in &provider.api_keys {
            self.api_key_index
                .insert(key.key.clone(), provider.id.clone());
        }
        self.providers.push(provider);
        Ok(())
    }

    pub fn update_provider_name(&mut self, id: &str, name: String) -> Result<(), CodexError> {
        let idx = self
            .provider_index
            .get(id)
            .copied()
            .ok_or_else(|| CodexError::NotFound(format!("Provider not found: {id}")))?;

        if name.trim().is_empty() {
            return Err(CodexError::Provider("Provider name cannot be empty".into()));
        }

        self.providers[idx].name = name;
        Ok(())
    }

    pub fn delete_provider(&mut self, id: &str) -> Result<(), CodexError> {
        let idx = self
            .provider_index
            .get(id)
            .copied()
            .ok_or_else(|| CodexError::NotFound(format!("Provider not found: {id}")))?;

        for key in &self.providers[idx].api_keys {
            self.api_key_index.remove(&key.key);
        }
        self.providers.remove(idx);
        self.provider_index.remove(id);
        self.rebuild_index();
        Ok(())
    }

    pub fn add_api_key(
        &mut self,
        provider_id: &str,
        api_key: String,
    ) -> Result<String, CodexError> {
        let idx = self
            .provider_index
            .get(provider_id)
            .copied()
            .ok_or_else(|| CodexError::NotFound(format!("Provider not found: {provider_id}")))?;

        if api_key.trim().is_empty() {
            return Err(CodexError::ApiKey("API key cannot be empty".into()));
        }

        if self.api_key_index.contains_key(&api_key) {
            return Err(CodexError::AlreadyExists("API key already exists".into()));
        }

        let key_id = generate_api_key_id();
        let key = CodexModelApiKey {
            id: key_id.clone(),
            key: api_key.clone(),
        };
        self.providers[idx].api_keys.push(key);
        self.api_key_index.insert(api_key, provider_id.to_string());
        Ok(key_id)
    }

    pub fn remove_api_key(&mut self, provider_id: &str, key_id: &str) -> Result<(), CodexError> {
        let idx = self
            .provider_index
            .get(provider_id)
            .copied()
            .ok_or_else(|| CodexError::NotFound(format!("Provider not found: {provider_id}")))?;

        let key_pos = self.providers[idx]
            .api_keys
            .iter()
            .position(|k| k.id == key_id)
            .ok_or_else(|| CodexError::NotFound(format!("API key not found: {key_id}")))?;

        let removed_key = self.providers[idx].api_keys.remove(key_pos);
        self.api_key_index.remove(&removed_key.key);
        Ok(())
    }

    pub fn cleanup_empty_preset_providers(&mut self) {
        self.providers.retain(|p| {
            if p.id.starts_with("preset_") && p.api_keys.is_empty() {
                self.provider_index.remove(&p.id);
                false
            } else {
                true
            }
        });
        self.rebuild_index();
    }

    pub fn import_provider(&mut self, provider: CodexModelProvider) -> Result<(), CodexError> {
        if self.find_by_id(&provider.id).is_some() {
            return Err(CodexError::AlreadyExists(format!(
                "Provider {} already exists",
                provider.id
            )));
        }
        self.add_provider(provider)
    }

    fn rebuild_index(&mut self) {
        self.provider_index.clear();
        self.api_key_index.clear();
        for (i, p) in self.providers.iter().enumerate() {
            self.provider_index.insert(p.id.clone(), i);
            for key in &p.api_keys {
                self.api_key_index.insert(key.key.clone(), p.id.clone());
            }
        }
    }
}

impl Default for ModelProviderStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_provider(id: &str, name: &str, base_url: &str) -> CodexModelProvider {
        CodexModelProvider {
            id: id.into(),
            name: name.into(),
            base_url: base_url.into(),
            api_keys: vec![],
        }
    }

    #[test]
    fn test_add_provider() {
        let mut store = ModelProviderStore::new();
        store
            .add_provider(sample_provider("cmp_test", "Test", "https://test.com/v1"))
            .unwrap();
        assert_eq!(store.providers().len(), 1);
    }

    #[test]
    fn test_duplicate_base_url_rejected() {
        let mut store = ModelProviderStore::new();
        store
            .add_provider(sample_provider("cmp_a", "A", "https://test.com/v1"))
            .unwrap();
        let result = store.add_provider(sample_provider("cmp_b", "B", "https://test.com/v1"));
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_provider_id_rejected() {
        let mut store = ModelProviderStore::new();
        store
            .add_provider(sample_provider("cmp_same", "A", "https://a.com/v1"))
            .unwrap();
        let result = store.add_provider(sample_provider("cmp_same", "B", "https://b.com/v1"));
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_provider_name_rejected() {
        let mut store = ModelProviderStore::new();
        let result = store.add_provider(sample_provider("cmp_e", "", "https://empty-name.com/v1"));
        assert!(result.is_err());
    }

    #[test]
    fn test_add_and_remove_api_key() {
        let mut store = ModelProviderStore::new();
        store
            .add_provider(sample_provider("cmp_test", "Test", "https://test.com/v1"))
            .unwrap();
        let key_id = store
            .add_api_key("cmp_test", "sk-test-key-123".into())
            .unwrap();
        assert!(!key_id.is_empty());

        let provider = store.find_by_id("cmp_test").unwrap();
        assert_eq!(provider.api_keys.len(), 1);

        store.remove_api_key("cmp_test", &key_id).unwrap();
        let provider = store.find_by_id("cmp_test").unwrap();
        assert_eq!(provider.api_keys.len(), 0);
    }

    #[test]
    fn test_duplicate_api_key_rejected() {
        let mut store = ModelProviderStore::new();
        store
            .add_provider(sample_provider("cmp_test", "Test", "https://test.com/v1"))
            .unwrap();
        store
            .add_api_key("cmp_test", "sk-duplicate".into())
            .unwrap();
        let result = store.add_api_key("cmp_test", "sk-duplicate".into());
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_provider_removes_keys() {
        let mut store = ModelProviderStore::new();
        store
            .add_provider(sample_provider("cmp_test", "Test", "https://test.com/v1"))
            .unwrap();
        store
            .add_api_key("cmp_test", "sk-to-delete".into())
            .unwrap();
        store.delete_provider("cmp_test").unwrap();
        assert!(store.providers().is_empty());
    }

    #[test]
    fn test_cleanup_empty_preset_providers() {
        let mut store = ModelProviderStore::new();
        store
            .add_provider(sample_provider(
                "preset_old_empty",
                "Old",
                "https://old.com/v1",
            ))
            .unwrap();
        store
            .add_provider(sample_provider("cmp_keep", "Keep", "https://keep.com/v1"))
            .unwrap();
        store.add_api_key("cmp_keep", "sk-keep".into()).unwrap();
        store.cleanup_empty_preset_providers();
        assert_eq!(store.providers().len(), 1);
        assert!(store.find_by_id("cmp_keep").is_some());
        assert!(store.find_by_id("preset_old_empty").is_none());
    }

    #[test]
    fn test_update_provider_name() {
        let mut store = ModelProviderStore::new();
        store
            .add_provider(sample_provider(
                "cmp_test",
                "Old Name",
                "https://test.com/v1",
            ))
            .unwrap();
        store
            .update_provider_name("cmp_test", "New Name".into())
            .unwrap();
        let p = store.find_by_id("cmp_test").unwrap();
        assert_eq!(p.name, "New Name");
    }

    #[test]
    fn test_from_providers_list() {
        let list = CodexModelProviderList {
            providers: vec![
                sample_provider("cmp_a", "A", "https://a.com/v1"),
                sample_provider("cmp_b", "B", "https://b.com/v1"),
            ],
        };
        let store = ModelProviderStore::from_providers(list.providers).unwrap();
        assert_eq!(store.providers().len(), 2);
    }
}
