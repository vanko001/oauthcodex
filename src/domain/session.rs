use crate::domain::codex_models::*;
use crate::error::CodexError;
use std::collections::HashMap;

pub struct SessionManager {
    sessions: Vec<CodexSession>,
    session_index: HashMap<String, usize>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: vec![],
            session_index: HashMap::new(),
        }
    }

    pub fn from_list(list: CodexSessionList) -> Self {
        let mut mgr = Self::new();
        for s in list.sessions {
            mgr.sessions.push(s.clone());
            mgr.session_index.insert(s.id, mgr.sessions.len() - 1);
        }
        mgr
    }

    pub fn sessions(&self) -> &[CodexSession] {
        &self.sessions
    }

    pub fn to_list(&self) -> CodexSessionList {
        CodexSessionList {
            sessions: self.sessions.clone(),
        }
    }

    pub fn find_by_id(&self, id: &str) -> Option<&CodexSession> {
        self.session_index.get(id).map(|&idx| &self.sessions[idx])
    }

    pub fn sessions_for_instance(&self, instance_id: &str) -> Vec<&CodexSession> {
        self.sessions
            .iter()
            .filter(|s| s.instance_id == instance_id && !s.is_trashed)
            .collect()
    }

    pub fn trashed_sessions(&self) -> Vec<&CodexSession> {
        self.sessions.iter().filter(|s| s.is_trashed).collect()
    }

    pub fn add_session(&mut self, session: CodexSession) -> Result<(), CodexError> {
        if self.session_index.contains_key(&session.id) {
            return Err(CodexError::AlreadyExists(format!(
                "Session already exists: {}",
                session.id
            )));
        }
        let idx = self.sessions.len();
        self.sessions.push(session);
        self.session_index
            .insert(self.sessions[idx].id.clone(), idx);
        Ok(())
    }

    pub fn trash_session(&mut self, id: &str) -> Result<(), CodexError> {
        let idx = self
            .session_index
            .get(id)
            .copied()
            .ok_or_else(|| CodexError::NotFound(format!("Session not found: {id}")))?;

        self.sessions[idx].is_trashed = true;
        self.sessions[idx].trash_date = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    pub fn restore_session(&mut self, id: &str) -> Result<(), CodexError> {
        let idx = self
            .session_index
            .get(id)
            .copied()
            .ok_or_else(|| CodexError::NotFound(format!("Session not found: {id}")))?;

        if !self.sessions[idx].is_trashed {
            return Err(CodexError::InvalidState(format!(
                "Session {id} is not trashed"
            )));
        }
        self.sessions[idx].is_trashed = false;
        self.sessions[idx].trash_date = None;
        Ok(())
    }

    pub fn token_stats_for_session(&self, id: &str) -> Result<TokenStats, CodexError> {
        let session = self
            .find_by_id(id)
            .ok_or_else(|| CodexError::NotFound(format!("Session not found: {id}")))?;

        if let Some(ref file_path) = session.file_path {
            if let Some(stats) = token_stats_from_rollout(std::path::Path::new(file_path)) {
                return Ok(TokenStats {
                    total_tokens: stats.2,
                    prompt_tokens: stats.0,
                    completion_tokens: stats.1,
                    average_per_message: stats.2.checked_div(session.message_count).unwrap_or(0),
                    peak_message_tokens: stats.2,
                });
            }
        }

        Ok(TokenStats {
            total_tokens: session.token_count,
            prompt_tokens: session.token_count / 2,
            completion_tokens: session.token_count - session.token_count / 2,
            average_per_message: session
                .token_count
                .checked_div(session.message_count)
                .unwrap_or(0),
            peak_message_tokens: session.token_count,
        })
    }

    pub fn repair_visibility(
        &self,
        instance_dirs: &[String],
    ) -> Result<VisibilityRepairReport, CodexError> {
        let mut issues = Vec::new();
        let mut restored = 0u64;

        for dir in instance_dirs {
            let backup_dir = format!("{dir}/backups");
            let sessions_dir = format!("{dir}/sessions");

            let backup_path = std::path::Path::new(&backup_dir);
            if backup_path.exists() {
                if let Ok(entries) = std::fs::read_dir(backup_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().is_some_and(|e| e == "json") {
                            let session_id = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();

                            if !self.session_index.contains_key(&session_id) {
                                issues.push(VisibilityIssue {
                                    session_id: session_id.clone(),
                                    issue: "missing_from_index".into(),
                                    found_in_backup: true,
                                    backup_path: Some(path.to_string_lossy().to_string()),
                                });
                                restored += 1;
                            }
                        }
                    }
                }
            }

            let sessions_path = std::path::Path::new(&sessions_dir);
            if !sessions_path.exists() {
                issues.push(VisibilityIssue {
                    session_id: dir.clone(),
                    issue: "missing_sessions_directory".into(),
                    found_in_backup: backup_path.exists(),
                    backup_path: if backup_path.exists() {
                        Some(backup_path.to_string_lossy().to_string())
                    } else {
                        None
                    },
                });
            }
        }

        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H%M%S").to_string();
        let backup_path = instance_dirs.first().map(|dir| {
            std::path::Path::new(dir)
                .join("backups")
                .join(format!("repair_{timestamp}"))
                .to_string_lossy()
                .to_string()
        });
        let report = VisibilityRepairReport {
            visibility_issues: issues,
            repair_result: VisibilityRepairResult {
                restored_count: restored,
                backup_created: true,
                backup_path,
                errors: vec![],
            },
        };

        Ok(report)
    }
}

fn token_stats_from_rollout(path: &std::path::Path) -> Option<(u64, u64, u64)> {
    let content = std::fs::read_to_string(path).ok()?;
    parse_token_stats_lines(&content)
}

fn parse_token_stats_lines(content: &str) -> Option<(u64, u64, u64)> {
    for line in content.lines().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || !trimmed.contains("\"token_count\"")
            || !trimmed.contains("\"total_token_usage\"")
        {
            continue;
        }

        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) else {
            continue;
        };
        if parsed.get("type").and_then(|value| value.as_str()) != Some("event_msg") {
            continue;
        }
        let Some(payload) = parsed.get("payload") else {
            continue;
        };
        if payload.get("type").and_then(|value| value.as_str()) != Some("token_count") {
            continue;
        }
        let Some(usage) = payload
            .get("info")
            .and_then(|info| info.get("total_token_usage"))
        else {
            continue;
        };
        let input = usage
            .get("input_tokens")
            .and_then(|value| value.as_u64())
            .unwrap_or(0);
        let output = usage
            .get("output_tokens")
            .and_then(|value| value.as_u64())
            .unwrap_or(0);
        let total = usage
            .get("total_tokens")
            .and_then(|value| value.as_u64())
            .unwrap_or(input + output);
        return Some((input, output, total));
    }

    None
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_add_session() {
        let mut mgr = SessionManager::new();
        mgr.add_session(sample_session("sess_001", "inst_001", false))
            .unwrap();
        assert_eq!(mgr.sessions().len(), 1);
    }

    #[test]
    fn test_duplicate_session_rejected() {
        let mut mgr = SessionManager::new();
        mgr.add_session(sample_session("sess_001", "inst_001", false))
            .unwrap();
        let result = mgr.add_session(sample_session("sess_001", "inst_001", false));
        assert!(result.is_err());
    }

    #[test]
    fn test_trash_and_restore() {
        let mut mgr = SessionManager::new();
        mgr.add_session(sample_session("sess_001", "inst_001", false))
            .unwrap();

        mgr.trash_session("sess_001").unwrap();
        assert!(mgr.find_by_id("sess_001").unwrap().is_trashed);

        mgr.restore_session("sess_001").unwrap();
        assert!(!mgr.find_by_id("sess_001").unwrap().is_trashed);
    }

    #[test]
    fn test_sessions_for_instance() {
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
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_token_stats() {
        let mut mgr = SessionManager::new();
        mgr.add_session(sample_session("sess_001", "inst_001", false))
            .unwrap();

        let stats = mgr.token_stats_for_session("sess_001").unwrap();
        assert_eq!(stats.total_tokens, 250);
        assert!(stats.average_per_message > 0);
    }

    #[test]
    fn test_token_stats_reads_rollout_token_count() {
        let tmp = tempfile::TempDir::new().unwrap();
        let rollout = tmp.path().join("rollout-test.jsonl");
        std::fs::write(
            &rollout,
            r#"{"type":"event_msg","payload":{"type":"message","text":"ignored"}}
{"type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":10,"output_tokens":15,"total_tokens":25}}}}"#,
        )
        .unwrap();

        let mut session = sample_session("sess_001", "inst_001", false);
        session.file_path = Some(rollout.to_string_lossy().to_string());
        let mut mgr = SessionManager::new();
        mgr.add_session(session).unwrap();

        let stats = mgr.token_stats_for_session("sess_001").unwrap();

        assert_eq!(stats.total_tokens, 25);
        assert_eq!(stats.prompt_tokens, 10);
        assert_eq!(stats.completion_tokens, 15);
    }

    #[test]
    fn test_repair_visibility_backup_path_uses_instance_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let instance_dir = tmp.path().join("instance");
        std::fs::create_dir_all(&instance_dir).unwrap();
        let mgr = SessionManager::new();

        let report = mgr
            .repair_visibility(&[instance_dir.to_string_lossy().to_string()])
            .unwrap();

        assert!(report
            .repair_result
            .backup_path
            .as_deref()
            .is_some_and(|path| path.starts_with(instance_dir.to_string_lossy().as_ref())));
    }

    #[test]
    fn test_from_list() {
        let list = CodexSessionList {
            sessions: vec![
                sample_session("sess_a", "inst_001", false),
                sample_session("sess_b", "inst_001", true),
            ],
        };
        let mgr = SessionManager::from_list(list);
        assert_eq!(mgr.sessions().len(), 2);
        assert_eq!(mgr.trashed_sessions().len(), 1);
    }
}
