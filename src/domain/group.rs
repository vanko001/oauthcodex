use crate::domain::codex_models::{CodexAccountGroup, CodexAccountGroupList};
use crate::error::CodexError;
use std::collections::HashMap;

pub struct GroupStore {
    groups: Vec<CodexAccountGroup>,
    group_index: HashMap<String, usize>,
    next_sort_order: u32,
}

impl GroupStore {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            group_index: HashMap::new(),
            next_sort_order: 0,
        }
    }

    pub fn from_groups(groups: Vec<CodexAccountGroup>) -> Self {
        let mut store = Self::new();
        for g in groups {
            store.groups.push(g.clone());
            store.group_index.insert(g.id, store.groups.len() - 1);
            store.next_sort_order = store.next_sort_order.max(g.sort_order + 1);
        }
        store
    }

    pub fn groups(&self) -> &[CodexAccountGroup] {
        &self.groups
    }

    pub fn to_list(&self) -> CodexAccountGroupList {
        CodexAccountGroupList {
            groups: self.groups.clone(),
        }
    }

    pub fn find_by_id(&self, id: &str) -> Option<&CodexAccountGroup> {
        self.group_index.get(id).map(|&idx| &self.groups[idx])
    }

    pub fn find_group_containing_account(&self, account_id: &str) -> Vec<&CodexAccountGroup> {
        self.groups
            .iter()
            .filter(|g| g.account_ids.contains(&account_id.to_string()))
            .collect()
    }

    pub fn generate_group_id() -> String {
        let uuid_str = uuid::Uuid::new_v4().to_string().replace('-', "_");
        format!("cgrp_{}", &uuid_str[..16])
    }

    pub fn create_group(&mut self, name: String) -> Result<String, CodexError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(CodexError::Group("Group name cannot be empty".into()));
        }

        let id = Self::generate_group_id();
        let sort_order = self.next_sort_order;
        self.next_sort_order += 1;

        self.groups.push(CodexAccountGroup {
            id: id.clone(),
            name: trimmed.to_string(),
            account_ids: vec![],
            sort_order,
        });
        self.group_index.insert(id.clone(), self.groups.len() - 1);
        Ok(id)
    }

    pub fn delete_group(&mut self, id: &str) -> Result<(), CodexError> {
        let idx = self
            .group_index
            .get(id)
            .copied()
            .ok_or_else(|| CodexError::NotFound(format!("Group not found: {id}")))?;

        self.groups.remove(idx);
        self.group_index.remove(id);
        self.rebuild_index();
        Ok(())
    }

    pub fn rename_group(&mut self, id: &str, name: String) -> Result<(), CodexError> {
        let idx = self
            .group_index
            .get(id)
            .copied()
            .ok_or_else(|| CodexError::NotFound(format!("Group not found: {id}")))?;

        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(CodexError::Group("Group name cannot be empty".into()));
        }

        self.groups[idx].name = trimmed.to_string();
        Ok(())
    }

    pub fn assign_accounts(
        &mut self,
        group_id: &str,
        account_ids: &[String],
    ) -> Result<(), CodexError> {
        let idx = self
            .group_index
            .get(group_id)
            .copied()
            .ok_or_else(|| CodexError::NotFound(format!("Group not found: {group_id}")))?;

        for account_id in account_ids {
            if !self.groups[idx].account_ids.contains(account_id) {
                self.groups[idx].account_ids.push(account_id.clone());
            }
            self.remove_account_from_other_groups(group_id, account_id);
        }
        Ok(())
    }

    pub fn remove_accounts(
        &mut self,
        group_id: &str,
        account_ids: &[String],
    ) -> Result<(), CodexError> {
        let idx = self
            .group_index
            .get(group_id)
            .copied()
            .ok_or_else(|| CodexError::NotFound(format!("Group not found: {group_id}")))?;

        let ids: std::collections::HashSet<&str> = account_ids.iter().map(|s| s.as_str()).collect();
        self.groups[idx]
            .account_ids
            .retain(|a| !ids.contains(a.as_str()));
        Ok(())
    }

    pub fn move_accounts(
        &mut self,
        target_group_id: &str,
        account_ids: &[String],
    ) -> Result<(), CodexError> {
        let target_idx = self
            .group_index
            .get(target_group_id)
            .copied()
            .ok_or_else(|| CodexError::NotFound(format!("Group not found: {target_group_id}")))?;

        for account_id in account_ids {
            if !self.groups[target_idx].account_ids.contains(account_id) {
                self.groups[target_idx].account_ids.push(account_id.clone());
            }
            self.remove_account_from_other_groups(target_group_id, account_id);
        }
        Ok(())
    }

    pub fn cleanup_deleted_accounts(&mut self, valid_account_ids: &[String]) {
        let valid: std::collections::HashSet<&str> =
            valid_account_ids.iter().map(|s| s.as_str()).collect();
        for group in &mut self.groups {
            group.account_ids.retain(|id| valid.contains(id.as_str()));
        }
    }

    fn remove_account_from_other_groups(&mut self, except_group_id: &str, account_id: &str) {
        for group in &mut self.groups {
            if group.id != except_group_id {
                group.account_ids.retain(|a| a != account_id);
            }
        }
    }

    pub fn import_group(&mut self, group: CodexAccountGroup) {
        let sort_order = self.next_sort_order;
        self.next_sort_order += 1;
        self.groups.push(CodexAccountGroup {
            id: group.id,
            name: group.name,
            account_ids: group.account_ids,
            sort_order,
        });
        self.group_index.insert(
            self.groups.last().unwrap().id.clone(),
            self.groups.len() - 1,
        );
    }

    fn rebuild_index(&mut self) {
        self.group_index.clear();
        for (i, g) in self.groups.iter().enumerate() {
            self.group_index.insert(g.id.clone(), i);
        }
    }
}

impl Default for GroupStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_group() {
        let mut store = GroupStore::new();
        let id = store.create_group("Work".into()).unwrap();
        assert!(id.starts_with("cgrp_"));
        assert_eq!(store.groups().len(), 1);
    }

    #[test]
    fn test_empty_group_name_rejected() {
        let mut store = GroupStore::new();
        let result = store.create_group("   ".into());
        assert!(result.is_err());
    }

    #[test]
    fn test_assign_and_move_accounts() {
        let mut store = GroupStore::new();
        let g1 = store.create_group("Group 1".into()).unwrap();
        let g2 = store.create_group("Group 2".into()).unwrap();

        store
            .assign_accounts(&g1, &["acct_a".into(), "acct_b".into()])
            .unwrap();
        let group1 = store.find_by_id(&g1).unwrap();
        assert_eq!(group1.account_ids.len(), 2);

        store.move_accounts(&g2, &["acct_a".into()]).unwrap();
        let group1 = store.find_by_id(&g1).unwrap();
        let group2 = store.find_by_id(&g2).unwrap();
        assert_eq!(group1.account_ids.len(), 1);
        assert_eq!(group2.account_ids.len(), 1);
        assert!(group2.account_ids.contains(&"acct_a".to_string()));
    }

    #[test]
    fn test_remove_accounts_from_group() {
        let mut store = GroupStore::new();
        let g1 = store.create_group("Group 1".into()).unwrap();
        store
            .assign_accounts(&g1, &["acct_a".into(), "acct_b".into(), "acct_c".into()])
            .unwrap();
        store
            .remove_accounts(&g1, &["acct_a".into(), "acct_c".into()])
            .unwrap();
        let group = store.find_by_id(&g1).unwrap();
        assert_eq!(group.account_ids, vec!["acct_b"]);
    }

    #[test]
    fn test_cleanup_deleted_accounts() {
        let mut store = GroupStore::new();
        let g1 = store.create_group("Group 1".into()).unwrap();
        store
            .assign_accounts(
                &g1,
                &["acct_a".into(), "acct_b".into(), "acct_deleted".into()],
            )
            .unwrap();
        store.cleanup_deleted_accounts(&["acct_a".into(), "acct_b".into()]);
        let group = store.find_by_id(&g1).unwrap();
        assert_eq!(group.account_ids.len(), 2);
        assert!(!group.account_ids.contains(&"acct_deleted".to_string()));
    }

    #[test]
    fn test_delete_group() {
        let mut store = GroupStore::new();
        let id = store.create_group("To Delete".into()).unwrap();
        store.delete_group(&id).unwrap();
        assert!(store.groups().is_empty());
    }

    #[test]
    fn test_rename_group() {
        let mut store = GroupStore::new();
        let id = store.create_group("Old Name".into()).unwrap();
        store.rename_group(&id, "New Name".into()).unwrap();
        let group = store.find_by_id(&id).unwrap();
        assert_eq!(group.name, "New Name");
    }

    #[test]
    fn test_from_groups_preserves_sort() {
        let groups = vec![
            CodexAccountGroup {
                id: "cgrp_a".into(),
                name: "A".into(),
                account_ids: vec![],
                sort_order: 2,
            },
            CodexAccountGroup {
                id: "cgrp_b".into(),
                name: "B".into(),
                account_ids: vec![],
                sort_order: 0,
            },
        ];
        let store = GroupStore::from_groups(groups);
        assert_eq!(store.groups().len(), 2);
        assert_eq!(store.find_by_id("cgrp_a").unwrap().sort_order, 2);
    }
}
