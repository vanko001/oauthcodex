use crate::domain::codex_models::*;
use crate::error::CodexError;
use std::collections::HashMap;

pub struct InstanceStore {
    instances: Vec<CodexInstance>,
    instance_index: HashMap<String, usize>,
}

impl InstanceStore {
    pub fn new() -> Self {
        Self {
            instances: vec![],
            instance_index: HashMap::new(),
        }
    }

    pub fn from_list(list: CodexInstanceList) -> Self {
        let mut store = Self::new();
        for inst in list.instances {
            store.instances.push(inst.clone());
            store
                .instance_index
                .insert(inst.id, store.instances.len() - 1);
        }
        store
    }

    pub fn instances(&self) -> &[CodexInstance] {
        &self.instances
    }

    pub fn to_list(&self) -> CodexInstanceList {
        CodexInstanceList {
            instances: self.instances.clone(),
        }
    }

    pub fn import_ref(&mut self, instance_ref: CodexInstanceStoreRef) -> Result<(), CodexError> {
        if self.find_by_id(&instance_ref.id).is_some() {
            return Err(CodexError::AlreadyExists(format!(
                "Instance {} already exists",
                instance_ref.id
            )));
        }

        if instance_ref.name.trim().is_empty() {
            return Err(CodexError::Instance("Instance name cannot be empty".into()));
        }

        if instance_ref.is_default {
            for instance in &mut self.instances {
                instance.is_default = false;
            }
        }

        let now = chrono::Utc::now().to_rfc3339();
        let instance = CodexInstance {
            id: instance_ref.id,
            name: instance_ref.name,
            is_default: instance_ref.is_default,
            working_dir: None,
            auth_mode: None,
            bound_account_id: None,
            follow_local_account: true,
            launch_mode: InstanceLaunchMode::Auto,
            extra_args: vec![],
            extra_env: HashMap::new(),
            enabled: true,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        };

        let idx = self.instances.len();
        self.instance_index.insert(instance.id.clone(), idx);
        self.instances.push(instance);
        Ok(())
    }

    pub fn find_by_id(&self, id: &str) -> Option<&CodexInstance> {
        self.instance_index.get(id).map(|&idx| &self.instances[idx])
    }

    pub fn default_instance(&self) -> Option<&CodexInstance> {
        self.instances.iter().find(|i| i.is_default)
    }

    pub fn create_instance(
        &mut self,
        name: String,
        is_default: bool,
        working_dir: Option<String>,
        _init_mode: &str,
    ) -> Result<String, CodexError> {
        if name.trim().is_empty() {
            return Err(CodexError::Instance("Instance name cannot be empty".into()));
        }

        let uuid_str = uuid::Uuid::new_v4().to_string().replace('-', "_");
        let id = format!("inst_codex_{}", &uuid_str[..16]);
        let now = chrono::Utc::now().to_rfc3339();

        let instance = CodexInstance {
            id: id.clone(),
            name: name.trim().to_string(),
            is_default,
            working_dir,
            auth_mode: None,
            bound_account_id: None,
            follow_local_account: true,
            launch_mode: InstanceLaunchMode::Auto,
            extra_args: vec![],
            extra_env: HashMap::new(),
            enabled: true,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        };

        let idx = self.instances.len();
        self.instances.push(instance);
        self.instance_index.insert(id.clone(), idx);
        Ok(id)
    }

    pub fn update_instance(&mut self, id: &str, updates: InstanceUpdate) -> Result<(), CodexError> {
        let idx = self
            .instance_index
            .get(id)
            .copied()
            .ok_or_else(|| CodexError::NotFound(format!("Instance not found: {id}")))?;

        let inst = &mut self.instances[idx];
        if let Some(name) = updates.name {
            if name.trim().is_empty() {
                return Err(CodexError::Instance("Instance name cannot be empty".into()));
            }
            inst.name = name;
        }
        if let Some(working_dir) = updates.working_dir {
            inst.working_dir = Some(working_dir);
        }
        if let Some(auth_mode) = updates.auth_mode {
            inst.auth_mode = Some(auth_mode);
        }
        if let Some(account_id) = updates.bound_account_id {
            inst.bound_account_id = Some(account_id);
        }
        if let Some(follow) = updates.follow_local_account {
            inst.follow_local_account = follow;
        }
        if let Some(launch_mode) = updates.launch_mode {
            inst.launch_mode = launch_mode;
        }
        if let Some(args) = updates.extra_args {
            inst.extra_args = args;
        }
        if let Some(env) = updates.extra_env {
            inst.extra_env = env;
        }
        inst.updated_at = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    pub fn delete_instance(&mut self, id: &str) -> Result<(), CodexError> {
        let idx = self
            .instance_index
            .get(id)
            .copied()
            .ok_or_else(|| CodexError::NotFound(format!("Instance not found: {id}")))?;

        if self.instances[idx].is_default {
            return Err(CodexError::Instance(
                "Cannot delete default instance".into(),
            ));
        }

        self.instances.remove(idx);
        self.instance_index.remove(id);
        self.rebuild_index();
        Ok(())
    }

    pub fn bind_account_to_default(&mut self, account_id: &str) -> Result<(), CodexError> {
        let default_idx = self
            .instances
            .iter()
            .position(|i| i.is_default)
            .ok_or_else(|| CodexError::NotFound("No default instance found".into()))?;

        self.instances[default_idx].bound_account_id = Some(account_id.to_string());
        self.instances[default_idx].updated_at = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    fn rebuild_index(&mut self) {
        self.instance_index.clear();
        for (i, inst) in self.instances.iter().enumerate() {
            self.instance_index.insert(inst.id.clone(), i);
        }
    }
}

impl Default for InstanceStore {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InstanceUpdate {
    pub name: Option<String>,
    pub working_dir: Option<String>,
    pub auth_mode: Option<CodexAuthMode>,
    pub bound_account_id: Option<String>,
    pub follow_local_account: Option<bool>,
    pub launch_mode: Option<InstanceLaunchMode>,
    pub extra_args: Option<Vec<String>>,
    pub extra_env: Option<HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn sample_default_instance() -> CodexInstance {
        let uuid_str = uuid::Uuid::new_v4().to_string().replace('-', "_");
        CodexInstance {
            id: format!("inst_codex_{}", &uuid_str[..16]),
            name: "Default".into(),
            is_default: true,
            working_dir: Some("/tmp/work".into()),
            auth_mode: None,
            bound_account_id: None,
            follow_local_account: true,
            launch_mode: InstanceLaunchMode::Auto,
            extra_args: vec![],
            extra_env: HashMap::new(),
            enabled: true,
            created_at: Some("2026-05-01T00:00:00Z".into()),
            updated_at: Some("2026-05-01T00:00:00Z".into()),
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
        assert!(store.default_instance().is_none());
    }

    #[test]
    fn test_empty_name_rejected() {
        let mut store = InstanceStore::new();
        let result = store.create_instance("  ".into(), false, None, "empty");
        assert!(result.is_err());
    }

    #[test]
    fn test_bind_account() {
        let mut store = InstanceStore::new();
        let id = store
            .create_instance("Default".into(), true, None, "empty")
            .unwrap();
        store.bind_account_to_default("acct_001").unwrap();
        let inst = store.find_by_id(&id).unwrap();
        assert_eq!(inst.bound_account_id, Some("acct_001".into()));
    }

    #[test]
    fn test_delete_named_instance() {
        let mut store = InstanceStore::new();
        let id = store
            .create_instance("Named".into(), false, None, "empty")
            .unwrap();
        store.delete_instance(&id).unwrap();
        assert!(store.instances().is_empty());
    }

    #[test]
    fn test_cannot_delete_default() {
        let mut store = InstanceStore::new();
        let id = store
            .create_instance("Default".into(), true, None, "empty")
            .unwrap();
        let result = store.delete_instance(&id);
        assert!(result.is_err());
    }
}
