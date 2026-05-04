use crate::adapters::fs_store::CodexPaths;
use crate::domain::codex_models::UserConfigCodex;
use crate::error::CodexError;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;
use toml_edit;

pub struct ConfigStore {
    pub paths: CodexPaths,
}

impl ConfigStore {
    pub fn new(paths: CodexPaths) -> Self {
        Self { paths }
    }

    pub fn load_config(&self) -> Result<UserConfigCodex, CodexError> {
        if !self.paths.config_file.exists() {
            return Ok(UserConfigCodex::default());
        }
        let content = fs::read_to_string(&self.paths.config_file).map_err(CodexError::Io)?;
        let doc: toml_edit::DocumentMut = content
            .parse()
            .map_err(|e| CodexError::Toml(format!("Failed to parse config TOML: {e}")))?;
        self.doc_to_config(&doc)
    }

    fn doc_to_config(&self, doc: &toml_edit::DocumentMut) -> Result<UserConfigCodex, CodexError> {
        let serialized = doc.to_string();
        let config: UserConfigCodex = toml::from_str(&serialized)
            .map_err(|e| CodexError::Toml(format!("Failed to deserialize config: {e}")))?;
        Ok(config)
    }

    pub fn save_config(&self, config: &UserConfigCodex) -> Result<(), CodexError> {
        if let Some(parent) = self.paths.config_file.parent() {
            fs::create_dir_all(parent).map_err(CodexError::Io)?;
        }

        let mut existing_doc = if self.paths.config_file.exists() {
            let content = fs::read_to_string(&self.paths.config_file).map_err(CodexError::Io)?;
            content
                .parse::<toml_edit::DocumentMut>()
                .map_err(|e| CodexError::Toml(format!("Failed to parse existing config: {e}")))?
        } else {
            toml_edit::DocumentMut::new()
        };

        let new_doc = self.config_to_doc(config)?;

        for key in Self::managed_keys() {
            if !new_doc.contains_key(key) {
                existing_doc.remove(key);
            }
        }

        for (key, value) in new_doc.iter() {
            existing_doc[key] = value.clone();
        }

        let tmp = NamedTempFile::new_in(
            self.paths
                .config_file
                .parent()
                .unwrap_or_else(|| std::path::Path::new("/tmp")),
        )
        .map_err(CodexError::Io)?;
        tmp.as_file()
            .write_all(existing_doc.to_string().as_bytes())
            .map_err(CodexError::Io)?;
        tmp.persist(&self.paths.config_file).map_err(|e| {
            CodexError::Io(std::io::Error::other(format!(
                "Atomic write failed for {}: {e}",
                self.paths.config_file.display()
            )))
        })?;

        Ok(())
    }

    fn config_to_doc(
        &self,
        config: &UserConfigCodex,
    ) -> Result<toml_edit::DocumentMut, CodexError> {
        let mut doc = toml_edit::DocumentMut::new();

        if let Some(v) = config.codex_auto_refresh_minutes {
            doc["codex_auto_refresh_minutes"] = toml_edit::value(v as i64);
        }
        if let Some(v) = config.codex_startup_wakeup_enabled {
            doc["codex_startup_wakeup_enabled"] = toml_edit::value(v);
        }
        if let Some(v) = config.codex_startup_wakeup_delay_seconds {
            doc["codex_startup_wakeup_delay_seconds"] = toml_edit::value(v as i64);
        }
        if let Some(ref v) = config.codex_app_path {
            doc["codex_app_path"] = toml_edit::value(v.as_str());
        }
        if let Some(ref v) = config.codex_specified_app_path {
            doc["codex_specified_app_path"] = toml_edit::value(v.as_str());
        }
        if let Some(v) = config.codex_launch_on_switch {
            doc["codex_launch_on_switch"] = toml_edit::value(v);
        }
        if let Some(v) = config.codex_restart_specified_app_on_switch {
            doc["codex_restart_specified_app_on_switch"] = toml_edit::value(v);
        }
        if let Some(v) = config.codex_local_access_entry_visible {
            doc["codex_local_access_entry_visible"] = toml_edit::value(v);
        }
        if let Some(v) = config.codex_auto_switch_enabled {
            doc["codex_auto_switch_enabled"] = toml_edit::value(v);
        }
        if let Some(v) = config.codex_auto_switch_primary_threshold {
            doc["codex_auto_switch_primary_threshold"] = toml_edit::value(v as i64);
        }
        if let Some(v) = config.codex_auto_switch_secondary_threshold {
            doc["codex_auto_switch_secondary_threshold"] = toml_edit::value(v as i64);
        }
        if let Some(ref v) = config.codex_auto_switch_account_scope_mode {
            doc["codex_auto_switch_account_scope_mode"] = toml_edit::value(v.as_str());
        }
        if let Some(ref v) = config.codex_auto_switch_selected_account_ids {
            doc["codex_auto_switch_selected_account_ids"] =
                toml_edit::value(string_array(v.iter().map(String::as_str)));
        }
        if let Some(v) = config.codex_quota_alert_enabled {
            doc["codex_quota_alert_enabled"] = toml_edit::value(v);
        }
        if let Some(v) = config.codex_quota_alert_threshold {
            doc["codex_quota_alert_threshold"] = toml_edit::value(v as i64);
        }
        if let Some(v) = config.codex_quota_alert_primary_threshold {
            doc["codex_quota_alert_primary_threshold"] = toml_edit::value(v as i64);
        }
        if let Some(v) = config.codex_quota_alert_secondary_threshold {
            doc["codex_quota_alert_secondary_threshold"] = toml_edit::value(v as i64);
        }
        if let Some(v) = config.codex_quota_alert_cooldown_minutes {
            doc["codex_quota_alert_cooldown_minutes"] = toml_edit::value(v as i64);
        }

        for (k, v) in &config.extra {
            if doc.contains_key(k) {
                continue;
            }
            match v {
                serde_json::Value::String(s) => {
                    doc[k.as_str()] = toml_edit::value(s.as_str());
                }
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        doc[k.as_str()] = toml_edit::value(i);
                    } else if let Some(f) = n.as_f64() {
                        doc[k.as_str()] = toml_edit::value(f);
                    }
                }
                serde_json::Value::Bool(b) => {
                    doc[k.as_str()] = toml_edit::value(*b);
                }
                serde_json::Value::Array(items) => {
                    if items.iter().all(|item| item.is_string()) {
                        doc[k.as_str()] = toml_edit::value(string_array(
                            items.iter().filter_map(|item| item.as_str()),
                        ));
                    }
                }
                _ => {}
            }
        }

        Ok(doc)
    }

    fn managed_keys() -> &'static [&'static str] {
        &[
            "codex_auto_refresh_minutes",
            "codex_startup_wakeup_enabled",
            "codex_startup_wakeup_delay_seconds",
            "codex_app_path",
            "codex_specified_app_path",
            "codex_launch_on_switch",
            "codex_restart_specified_app_on_switch",
            "codex_local_access_entry_visible",
            "codex_auto_switch_enabled",
            "codex_auto_switch_primary_threshold",
            "codex_auto_switch_secondary_threshold",
            "codex_auto_switch_account_scope_mode",
            "codex_auto_switch_selected_account_ids",
            "codex_quota_alert_enabled",
            "codex_quota_alert_threshold",
            "codex_quota_alert_primary_threshold",
            "codex_quota_alert_secondary_threshold",
            "codex_quota_alert_cooldown_minutes",
        ]
    }

    pub fn set_codex_auto_refresh(&self, minutes: i32) -> Result<(), CodexError> {
        let mut config = self.load_config()?;
        let clamped = if minutes == -1 {
            -1
        } else {
            minutes.clamp(1, 1440)
        };
        config.codex_auto_refresh_minutes = Some(clamped);
        self.save_config(&config)
    }

    pub fn set_codex_launch_on_switch(&self, enabled: bool) -> Result<(), CodexError> {
        let mut config = self.load_config()?;
        config.codex_launch_on_switch = Some(enabled);
        self.save_config(&config)
    }

    pub fn set_codex_local_access_entry_visible(&self, visible: bool) -> Result<(), CodexError> {
        let mut config = self.load_config()?;
        config.codex_local_access_entry_visible = Some(visible);
        self.save_config(&config)
    }

    pub fn set_codex_app_path(&self, path: &str) -> Result<(), CodexError> {
        let mut config = self.load_config()?;
        config.codex_app_path = if path.is_empty() {
            None
        } else {
            Some(path.to_string())
        };
        self.save_config(&config)
    }

    pub fn set_codex_auto_switch(
        &self,
        enabled: bool,
        primary_threshold: Option<i32>,
        secondary_threshold: Option<i32>,
    ) -> Result<(), CodexError> {
        let mut config = self.load_config()?;
        config.codex_auto_switch_enabled = Some(enabled);
        config.codex_auto_switch_primary_threshold = primary_threshold.map(|v| v.clamp(0, 100));
        config.codex_auto_switch_secondary_threshold = secondary_threshold.map(|v| v.clamp(0, 100));
        config
            .codex_auto_switch_account_scope_mode
            .get_or_insert_with(|| "all_accounts".to_string());
        config
            .codex_auto_switch_selected_account_ids
            .get_or_insert_with(Vec::new);
        self.save_config(&config)
    }

    pub fn set_codex_quota_alert(
        &self,
        enabled: bool,
        primary_threshold: Option<i32>,
    ) -> Result<(), CodexError> {
        let mut config = self.load_config()?;
        config.codex_quota_alert_enabled = Some(enabled);
        config.codex_quota_alert_threshold = primary_threshold.map(|v| v.clamp(0, 100));
        config.codex_quota_alert_primary_threshold = primary_threshold.map(|v| v.clamp(0, 100));
        self.save_config(&config)
    }
}

fn string_array<'a>(items: impl IntoIterator<Item = &'a str>) -> toml_edit::Array {
    let mut array = toml_edit::Array::new();
    for item in items {
        array.push(item);
    }
    array
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn setup_test_store() -> (ConfigStore, TempDir) {
        let tmp = TempDir::new().expect("temp dir");
        let paths = CodexPaths::for_tests(tmp.path());
        let store = ConfigStore::new(paths);
        (store, tmp)
    }

    #[test]
    fn test_load_default_config() {
        let (store, _tmp) = setup_test_store();
        let config = store.load_config().expect("load config");
        assert_eq!(config.codex_auto_refresh_minutes, Some(30));
        assert_eq!(config.codex_launch_on_switch, None);
        assert!(config.extra.is_empty());
    }

    #[test]
    fn test_set_auto_refresh_valid() {
        let (store, _tmp) = setup_test_store();
        store.set_codex_auto_refresh(60).expect("set refresh");
        let config = store.load_config().expect("load config");
        assert_eq!(config.codex_auto_refresh_minutes, Some(60));
    }

    #[test]
    fn test_set_auto_refresh_negative_one() {
        let (store, _tmp) = setup_test_store();
        store.set_codex_auto_refresh(-1).expect("set refresh -1");
        let config = store.load_config().expect("load config");
        assert_eq!(config.codex_auto_refresh_minutes, Some(-1));
    }

    #[test]
    fn test_set_auto_refresh_zero_clamped_to_one() {
        let (store, _tmp) = setup_test_store();
        store.set_codex_auto_refresh(0).expect("set refresh 0");
        let config = store.load_config().expect("load config");
        assert_eq!(config.codex_auto_refresh_minutes, Some(1));
    }

    #[test]
    fn test_set_auto_refresh_above_max_clamped() {
        let (store, _tmp) = setup_test_store();
        store
            .set_codex_auto_refresh(9999)
            .expect("set refresh 9999");
        let config = store.load_config().expect("load config");
        assert_eq!(config.codex_auto_refresh_minutes, Some(1440));
    }

    #[test]
    fn test_set_launch_on_switch() {
        let (store, _tmp) = setup_test_store();
        store
            .set_codex_launch_on_switch(true)
            .expect("set launch on switch");
        let config = store.load_config().expect("load config");
        assert_eq!(config.codex_launch_on_switch, Some(true));

        store
            .set_codex_launch_on_switch(false)
            .expect("set launch off");
        let config = store.load_config().expect("load config");
        assert_eq!(config.codex_launch_on_switch, Some(false));
    }

    #[test]
    fn test_set_local_access_entry_visible() {
        let (store, _tmp) = setup_test_store();
        store
            .set_codex_local_access_entry_visible(true)
            .expect("set visible");
        let config = store.load_config().expect("load config");
        assert_eq!(config.codex_local_access_entry_visible, Some(true));
    }

    #[test]
    fn test_set_app_path() {
        let (store, _tmp) = setup_test_store();
        store
            .set_codex_app_path("/usr/local/bin/codex")
            .expect("set app path");
        let config = store.load_config().expect("load config");
        assert_eq!(
            config.codex_app_path,
            Some("/usr/local/bin/codex".to_string())
        );

        store.set_codex_app_path("").expect("clear app path");
        let config = store.load_config().expect("load config");
        assert_eq!(config.codex_app_path, None);
    }

    #[test]
    fn test_set_auto_switch_thresholds() {
        let (store, _tmp) = setup_test_store();
        store
            .set_codex_auto_switch(true, Some(80), Some(20))
            .expect("set auto switch");
        let config = store.load_config().expect("load config");
        assert_eq!(config.codex_auto_switch_enabled, Some(true));
        assert_eq!(config.codex_auto_switch_primary_threshold, Some(80));
        assert_eq!(config.codex_auto_switch_secondary_threshold, Some(20));
        assert_eq!(
            config.codex_auto_switch_account_scope_mode.as_deref(),
            Some("all_accounts")
        );
    }

    #[test]
    fn test_set_auto_switch_threshold_clamping() {
        let (store, _tmp) = setup_test_store();
        store
            .set_codex_auto_switch(true, Some(-5), Some(200))
            .expect("set auto switch");
        let config = store.load_config().expect("load config");
        assert_eq!(config.codex_auto_switch_primary_threshold, Some(0));
        assert_eq!(config.codex_auto_switch_secondary_threshold, Some(100));
    }

    #[test]
    fn test_set_quota_alert_thresholds() {
        let (store, _tmp) = setup_test_store();
        store
            .set_codex_quota_alert(true, Some(75))
            .expect("set quota alert");
        let config = store.load_config().expect("load config");
        assert_eq!(config.codex_quota_alert_enabled, Some(true));
        assert_eq!(config.codex_quota_alert_threshold, Some(75));
        assert_eq!(config.codex_quota_alert_primary_threshold, Some(75));
    }

    #[test]
    fn test_set_quota_alert_threshold_clamping() {
        let (store, _tmp) = setup_test_store();
        store
            .set_codex_quota_alert(true, Some(150))
            .expect("set quota alert");
        let config = store.load_config().expect("load config");
        assert_eq!(config.codex_quota_alert_threshold, Some(100));
        assert_eq!(config.codex_quota_alert_primary_threshold, Some(100));
    }

    #[test]
    fn test_roundtrip_config_with_extra_fields_preserved() {
        let (store, _tmp) = setup_test_store();

        let mut config = UserConfigCodex {
            codex_auto_refresh_minutes: Some(45),
            codex_launch_on_switch: Some(true),
            ..Default::default()
        };

        let mut extra = HashMap::new();
        extra.insert(
            "other_provider_field".to_string(),
            serde_json::Value::String("should_be_preserved".to_string()),
        );
        extra.insert(
            "custom_database_path".to_string(),
            serde_json::Value::String("/tmp/custom.db".to_string()),
        );
        config.extra = extra;

        store.save_config(&config).expect("save config");

        let loaded = store.load_config().expect("load config");
        assert_eq!(loaded.codex_auto_refresh_minutes, Some(45));
        assert_eq!(loaded.codex_launch_on_switch, Some(true));

        assert_eq!(
            loaded
                .extra
                .get("other_provider_field")
                .and_then(|v| v.as_str()),
            Some("should_be_preserved")
        );
        assert_eq!(
            loaded
                .extra
                .get("custom_database_path")
                .and_then(|v| v.as_str()),
            Some("/tmp/custom.db")
        );
    }
}
