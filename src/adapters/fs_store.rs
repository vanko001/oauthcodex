use crate::error::CodexError;
use serde::de::Error as _;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

#[derive(Clone)]
pub struct CodexPaths {
    pub home: PathBuf,
    pub codex_dir: PathBuf,
    pub cockpit_dir: PathBuf,
    pub auth_file: PathBuf,
    pub config_file: PathBuf,
    pub account_index_file: PathBuf,
    pub account_dir: PathBuf,
    pub codex_account_groups_file: PathBuf,
    pub codex_model_providers_file: PathBuf,
    pub codex_oauth_pending_file: PathBuf,
    pub codex_local_access_file: PathBuf,
    pub codex_local_access_stats_file: PathBuf,
}

impl CodexPaths {
    pub fn new() -> Result<Self, CodexError> {
        let home = dirs_home()?;
        let codex_dir = codex_home_from_env().unwrap_or_else(|| home.join(".codex"));
        let cockpit_dir = home.join(".antigravity_cockpit");
        let account_dir = cockpit_dir.join("codex_accounts");

        Ok(Self {
            auth_file: codex_dir.join("auth.json"),
            config_file: codex_dir.join("config.toml"),
            codex_dir,
            account_dir,
            account_index_file: cockpit_dir.join("codex_accounts.json"),
            codex_account_groups_file: cockpit_dir.join("codex_account_groups.json"),
            codex_model_providers_file: cockpit_dir.join("codex_model_providers.json"),
            codex_oauth_pending_file: cockpit_dir.join("codex_oauth_pending.json"),
            codex_local_access_file: cockpit_dir.join("codex_local_access.json"),
            codex_local_access_stats_file: cockpit_dir.join("codex_local_access_stats.json"),
            cockpit_dir,
            home,
        })
    }

    pub fn for_tests(tmp: &Path) -> Self {
        let home = tmp.to_path_buf();
        let codex_dir = home.join(".codex");
        let cockpit_dir = home.join(".antigravity_cockpit");
        let account_dir = cockpit_dir.join("codex_accounts");

        Self {
            auth_file: codex_dir.join("auth.json"),
            config_file: codex_dir.join("config.toml"),
            codex_dir,
            account_dir,
            account_index_file: cockpit_dir.join("codex_accounts.json"),
            codex_account_groups_file: cockpit_dir.join("codex_account_groups.json"),
            codex_model_providers_file: cockpit_dir.join("codex_model_providers.json"),
            codex_oauth_pending_file: cockpit_dir.join("codex_oauth_pending.json"),
            codex_local_access_file: cockpit_dir.join("codex_local_access.json"),
            codex_local_access_stats_file: cockpit_dir.join("codex_local_access_stats.json"),
            cockpit_dir,
            home,
        }
    }

    pub fn account_file(&self, account_id: &str) -> PathBuf {
        self.account_dir.join(format!("{}.json", account_id))
    }

    pub fn ensure_dirs(&self) -> Result<(), CodexError> {
        fs::create_dir_all(&self.codex_dir).map_err(CodexError::Io)?;
        fs::create_dir_all(&self.cockpit_dir).map_err(CodexError::Io)?;
        fs::create_dir_all(&self.account_dir).map_err(CodexError::Io)?;
        Ok(())
    }
}

fn codex_home_from_env() -> Option<PathBuf> {
    let raw = std::env::var("CODEX_HOME").ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let unquoted = trimmed.trim_matches('"').trim_matches('\'').trim();
    if unquoted.is_empty() {
        None
    } else {
        Some(PathBuf::from(unquoted))
    }
}

fn dirs_home() -> Result<PathBuf, CodexError> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .or_else(|_| {
            #[cfg(target_os = "macos")]
            {
                std::env::var("HOME").map(PathBuf::from)
            }
            #[cfg(not(target_os = "macos"))]
            {
                dirs_sys_alt()
            }
        })
        .map_err(|_| CodexError::Config("Cannot determine home directory".into()))
}

#[cfg(not(target_os = "macos"))]
fn dirs_sys_alt() -> Result<PathBuf, CodexError> {
    Err(CodexError::Config("Cannot determine home directory".into()))
}

pub fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, CodexError> {
    if !path.exists() {
        return Err(CodexError::NotFound(format!(
            "File not found: {}",
            path.display()
        )));
    }
    let content = fs::read_to_string(path).map_err(|e| {
        CodexError::Io(if e.kind() == std::io::ErrorKind::NotFound {
            std::io::Error::new(
                e.kind(),
                format!("File not found: {} ({e})", path.display()),
            )
        } else {
            e
        })
    })?;
    if content.trim().is_empty() {
        return Err(CodexError::Json(serde_json::Error::custom(format!(
            "Empty file: {}",
            path.display()
        ))));
    }
    serde_json::from_str(&content).map_err(|e| {
        CodexError::Json(serde_json::Error::custom(format!(
            "Invalid JSON in {}: {e}",
            path.display()
        )))
    })
}

pub fn read_json_file_opt<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> Result<Option<T>, CodexError> {
    if !path.exists() {
        return Ok(None);
    }
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(CodexError::Io(e)),
    };
    if content.trim().is_empty() {
        return Ok(None);
    }
    serde_json::from_str(&content)
        .map(Some)
        .map_err(CodexError::Json)
}

pub fn write_json_atomic<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), CodexError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(CodexError::Io)?;
    }
    let data = serde_json::to_string_pretty(value)?;
    write_string_atomic(path, &data)
}

pub fn write_string_atomic(path: &Path, data: &str) -> Result<(), CodexError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(CodexError::Io)?;
    }
    let tmp = NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new("/tmp")))
        .map_err(CodexError::Io)?;
    fs::write(tmp.path(), data).map_err(CodexError::Io)?;
    tmp.persist(path).map_err(|e| {
        CodexError::Io(std::io::Error::other(format!(
            "Atomic write failed for {}: {e}",
            path.display()
        )))
    })?;
    Ok(())
}

pub fn write_json_atomic_in_dir(
    dir: &Path,
    filename: &str,
    value: &impl serde::Serialize,
) -> Result<(), CodexError> {
    write_json_atomic(&dir.join(filename), value)
}
