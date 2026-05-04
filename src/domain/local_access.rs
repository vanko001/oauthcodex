use crate::adapters::fs_store::{read_json_file_opt, write_json_atomic, CodexPaths};
use crate::domain::codex_models::*;
use crate::error::CodexError;
use chrono::Utc;
use rand::Rng;

const DEFAULT_VERSION: u32 = 1;
const MAX_STATS_EVENTS: usize = 1000;

pub struct LocalAccessService {
    pub paths: CodexPaths,
}

impl LocalAccessService {
    pub fn new(paths: CodexPaths) -> Self {
        Self { paths }
    }

    pub fn load_collection(&self) -> Result<LocalAccessCollection, CodexError> {
        match read_json_file_opt(&self.paths.codex_local_access_file) {
            Ok(Some(collection)) => Ok(collection),
            Ok(None) => {
                let default = LocalAccessCollection {
                    version: DEFAULT_VERSION,
                    accounts: vec![],
                    port: 0,
                    enabled: false,
                    local_api_key: Self::generate_local_api_key(),
                    restrict_free_accounts: false,
                    routing_strategy: RoutingStrategy::Auto,
                    created_at: Some(Utc::now().to_rfc3339()),
                    updated_at: None,
                };
                write_json_atomic(&self.paths.codex_local_access_file, &default)?;
                Ok(default)
            }
            Err(e) => {
                if let CodexError::Json(_) = &e {
                    let default = LocalAccessCollection {
                        version: DEFAULT_VERSION,
                        accounts: vec![],
                        port: 0,
                        enabled: false,
                        local_api_key: Self::generate_local_api_key(),
                        restrict_free_accounts: false,
                        routing_strategy: RoutingStrategy::Auto,
                        created_at: Some(Utc::now().to_rfc3339()),
                        updated_at: None,
                    };
                    write_json_atomic(&self.paths.codex_local_access_file, &default)?;
                    Ok(default)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn save_collection(&self, collection: &LocalAccessCollection) -> Result<(), CodexError> {
        write_json_atomic(&self.paths.codex_local_access_file, collection)
    }

    pub fn generate_local_api_key() -> String {
        let random_hex: String = (0..32)
            .map(|_| {
                let nibble: u8 = rand::thread_rng().gen_range(0..16);
                char::from_digit(nibble as u32, 16).unwrap()
            })
            .collect();
        format!("sk-local-{}", random_hex)
    }

    pub fn sanitize_collection(
        &self,
        collection: &mut LocalAccessCollection,
        valid_oauth_accounts: &[CodexAccount],
    ) {
        let valid_ids: std::collections::HashSet<&str> = valid_oauth_accounts
            .iter()
            .filter(|a| a.is_oauth())
            .map(|a| a.id.as_str())
            .collect();

        if collection.restrict_free_accounts {
            let free_ids: std::collections::HashSet<&str> = valid_oauth_accounts
                .iter()
                .filter(|a| {
                    a.is_oauth()
                        && a.plan_type
                            .as_ref()
                            .is_some_and(|p| p.to_lowercase() == "free")
                })
                .map(|a| a.id.as_str())
                .collect();

            collection
                .accounts
                .retain(|id| valid_ids.contains(id.as_str()) && !free_ids.contains(id.as_str()));
        } else {
            collection
                .accounts
                .retain(|id| valid_ids.contains(id.as_str()));
        }
    }

    pub fn get_state_snapshot(
        &self,
        collection: &LocalAccessCollection,
        running: bool,
    ) -> LocalAccessStateSnapshot {
        let stats = match read_json_file_opt::<LocalAccessStatsFile>(
            &self.paths.codex_local_access_stats_file,
        ) {
            Ok(Some(stats_file)) => {
                let mut daily = LocalAccessStatsWindow {
                    requests: 0,
                    successes: 0,
                    failures: 0,
                    tokens_in: 0,
                    tokens_out: 0,
                    latency_ms_sum: 0,
                };
                let mut weekly = LocalAccessStatsWindow {
                    requests: 0,
                    successes: 0,
                    failures: 0,
                    tokens_in: 0,
                    tokens_out: 0,
                    latency_ms_sum: 0,
                };
                let mut monthly = LocalAccessStatsWindow {
                    requests: 0,
                    successes: 0,
                    failures: 0,
                    tokens_in: 0,
                    tokens_out: 0,
                    latency_ms_sum: 0,
                };

                let now = Utc::now();
                for event in &stats_file.requests {
                    let event_time = chrono::DateTime::parse_from_rfc3339(&event.timestamp)
                        .map(|t| t.with_timezone(&Utc))
                        .unwrap_or(now);

                    let age = now.signed_duration_since(event_time);
                    let is_success = event.status >= 200 && event.status < 300;

                    let window = if age.num_hours() < 24 {
                        Some(&mut daily)
                    } else if age.num_days() < 7 {
                        Some(&mut weekly)
                    } else if age.num_days() < 30 {
                        Some(&mut monthly)
                    } else {
                        None
                    };

                    if let Some(w) = window {
                        w.requests += 1;
                        if is_success {
                            w.successes += 1;
                        } else {
                            w.failures += 1;
                        }
                        w.tokens_in += event.tokens_in;
                        w.tokens_out += event.tokens_out;
                        w.latency_ms_sum += event.latency_ms;
                    }
                }

                Some(LocalAccessStatsSnapshot {
                    daily,
                    weekly,
                    monthly,
                })
            }
            _ => None,
        };

        LocalAccessStateSnapshot {
            enabled: collection.enabled,
            running,
            port: collection.port,
            base_url: format!("http://localhost:{}", collection.port),
            account_count: collection.accounts.len(),
            last_error: None,
            local_api_key: collection.local_api_key.clone(),
            stats,
        }
    }

    pub fn rotate_api_key(
        &self,
        collection: &mut LocalAccessCollection,
    ) -> Result<String, CodexError> {
        let new_key = Self::generate_local_api_key();
        collection.local_api_key = new_key.clone();
        collection.updated_at = Some(Utc::now().to_rfc3339());
        self.save_collection(collection)?;
        Ok(new_key)
    }

    pub fn update_routing(
        &self,
        collection: &mut LocalAccessCollection,
        strategy: RoutingStrategy,
    ) -> Result<(), CodexError> {
        collection.routing_strategy = strategy;
        collection.updated_at = Some(Utc::now().to_rfc3339());
        self.save_collection(collection)
    }

    pub fn set_enabled(
        &self,
        collection: &mut LocalAccessCollection,
        enabled: bool,
    ) -> Result<(), CodexError> {
        collection.enabled = enabled;
        collection.updated_at = Some(Utc::now().to_rfc3339());
        self.save_collection(collection)
    }

    pub fn update_port(
        &self,
        collection: &mut LocalAccessCollection,
        port: u16,
    ) -> Result<(), CodexError> {
        collection.port = port;
        collection.updated_at = Some(Utc::now().to_rfc3339());
        self.save_collection(collection)
    }

    pub fn save_accounts(
        &self,
        collection: &mut LocalAccessCollection,
        account_ids: &[String],
    ) -> Result<(), CodexError> {
        collection.accounts = account_ids.to_vec();
        collection.updated_at = Some(Utc::now().to_rfc3339());
        self.save_collection(collection)
    }

    pub fn clear_stats(&self) -> Result<(), CodexError> {
        if self.paths.codex_local_access_stats_file.exists() {
            std::fs::remove_file(&self.paths.codex_local_access_stats_file)
                .map_err(CodexError::Io)?;
        }
        Ok(())
    }

    pub fn record_request(&self, event: LocalAccessStatsEvent) -> Result<(), CodexError> {
        let mut stats: LocalAccessStatsFile =
            match read_json_file_opt(&self.paths.codex_local_access_stats_file) {
                Ok(Some(f)) => f,
                _ => LocalAccessStatsFile { requests: vec![] },
            };

        stats.requests.push(event);

        if stats.requests.len() > MAX_STATS_EVENTS {
            let excess = stats.requests.len() - MAX_STATS_EVENTS;
            stats.requests.drain(0..excess);
        }

        write_json_atomic(&self.paths.codex_local_access_stats_file, &stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup(tmp: &TempDir) -> (LocalAccessService, LocalAccessCollection) {
        let paths = CodexPaths::for_tests(tmp.path());
        std::fs::create_dir_all(&paths.cockpit_dir).unwrap();
        let svc = LocalAccessService::new(paths);
        let collection = svc.load_collection().unwrap();
        (svc, collection)
    }

    fn make_oauth_account(id: &str, plan: &str) -> CodexAccount {
        CodexAccount {
            id: id.to_string(),
            provider: "openai".to_string(),
            auth_mode: CodexAuthMode::OAuth,
            email: Some(format!("{}@example.com", id)),
            plan_type: Some(plan.to_string()),
            account_id: Some(id.to_string()),
            organization_id: None,
            organizations: vec![],
            display_name: id.to_string(),
            tags: vec![],
            tokens: CodexTokens::empty(),
            api_key: None,
            base_url: Some("https://api.openai.com/v1".to_string()),
            provider_id: Some("openai".to_string()),
            provider_name: Some("OpenAI".to_string()),
            api_provider_mode: Some(CodexApiProviderMode::OpenAI),
            quota: None,
            created_at: Some(Utc::now().to_rfc3339()),
            last_used: None,
            last_refresh: None,
        }
    }

    fn make_apikey_account(id: &str) -> CodexAccount {
        CodexAccount {
            id: id.to_string(),
            provider: "openai".to_string(),
            auth_mode: CodexAuthMode::ApiKey,
            email: Some(format!("{}@example.com", id)),
            plan_type: Some("pro".to_string()),
            account_id: Some(id.to_string()),
            organization_id: None,
            organizations: vec![],
            display_name: id.to_string(),
            tags: vec![],
            tokens: CodexTokens::empty(),
            api_key: Some("sk-proj-abc123".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            provider_id: Some("openai".to_string()),
            provider_name: Some("OpenAI".to_string()),
            api_provider_mode: Some(CodexApiProviderMode::OpenAI),
            quota: None,
            created_at: Some(Utc::now().to_rfc3339()),
            last_used: None,
            last_refresh: None,
        }
    }

    #[test]
    fn test_create_collection() {
        let tmp = TempDir::new().unwrap();
        let (svc, collection) = setup(&tmp);

        assert_eq!(collection.version, 1);
        assert!(collection.accounts.is_empty());
        assert_eq!(collection.port, 0);
        assert!(!collection.enabled);
        assert!(collection.local_api_key.starts_with("sk-local-"));
        assert!(!collection.restrict_free_accounts);
        assert_eq!(collection.routing_strategy, RoutingStrategy::Auto);

        let loaded = svc.load_collection().unwrap();
        assert_eq!(collection.local_api_key, loaded.local_api_key);
    }

    #[test]
    fn test_add_accounts() {
        let tmp = TempDir::new().unwrap();
        let (svc, mut collection) = setup(&tmp);

        svc.save_accounts(
            &mut collection,
            &["acct_a".to_string(), "acct_b".to_string()],
        )
        .unwrap();

        let loaded = svc.load_collection().unwrap();
        assert_eq!(loaded.accounts.len(), 2);
        assert_eq!(loaded.accounts[0], "acct_a");
        assert_eq!(loaded.accounts[1], "acct_b");
        assert!(loaded.updated_at.is_some());
    }

    #[test]
    fn test_rotate_key() {
        let tmp = TempDir::new().unwrap();
        let (svc, mut collection) = setup(&tmp);

        let old_key = collection.local_api_key.clone();
        let new_key = svc.rotate_api_key(&mut collection).unwrap();

        assert_ne!(old_key, new_key);
        assert!(new_key.starts_with("sk-local-"));
        assert_eq!(new_key.len(), "sk-local-".len() + 32);

        let loaded = svc.load_collection().unwrap();
        assert_eq!(loaded.local_api_key, new_key);
    }

    #[test]
    fn test_update_routing() {
        let tmp = TempDir::new().unwrap();
        let (svc, mut collection) = setup(&tmp);

        svc.update_routing(&mut collection, RoutingStrategy::RoundRobin)
            .unwrap();
        assert_eq!(collection.routing_strategy, RoutingStrategy::RoundRobin);

        let loaded = svc.load_collection().unwrap();
        assert_eq!(loaded.routing_strategy, RoutingStrategy::RoundRobin);
    }

    #[test]
    fn test_state_snapshot() {
        let tmp = TempDir::new().unwrap();
        let (svc, mut collection) = setup(&tmp);

        collection.enabled = true;
        collection.port = 8080;
        svc.save_accounts(&mut collection, &["acct_a".to_string()])
            .unwrap();

        let snapshot = svc.get_state_snapshot(&collection, true);

        assert!(snapshot.enabled);
        assert!(snapshot.running);
        assert_eq!(snapshot.port, 8080);
        assert_eq!(snapshot.base_url, "http://localhost:8080");
        assert_eq!(snapshot.account_count, 1);
        assert_eq!(snapshot.local_api_key, collection.local_api_key);
        assert!(snapshot.last_error.is_none());
    }

    #[test]
    fn test_sanitize_stale_accounts() {
        let tmp = TempDir::new().unwrap();
        let (svc, mut collection) = setup(&tmp);

        collection.accounts = vec![
            "acct_oauth_pro_001".to_string(),
            "acct_missing_deleted".to_string(),
        ];

        let valid = vec![make_oauth_account("acct_oauth_pro_001", "pro")];
        svc.sanitize_collection(&mut collection, &valid);

        assert_eq!(collection.accounts.len(), 1);
        assert_eq!(collection.accounts[0], "acct_oauth_pro_001");
    }

    #[test]
    fn test_sanitize_reject_apikey_accounts() {
        let tmp = TempDir::new().unwrap();
        let (svc, mut collection) = setup(&tmp);

        collection.accounts = vec!["acct_oauth_001".to_string(), "acct_apikey_001".to_string()];

        let valid = vec![
            make_oauth_account("acct_oauth_001", "pro"),
            make_apikey_account("acct_apikey_001"),
        ];
        svc.sanitize_collection(&mut collection, &valid);

        assert_eq!(collection.accounts.len(), 1);
        assert_eq!(collection.accounts[0], "acct_oauth_001");
    }

    #[test]
    fn test_sanitize_reject_free_accounts_when_restricted() {
        let tmp = TempDir::new().unwrap();
        let (svc, mut collection) = setup(&tmp);

        collection.restrict_free_accounts = true;
        collection.accounts = vec![
            "acct_pro".to_string(),
            "acct_free".to_string(),
            "acct_team".to_string(),
        ];

        let valid = vec![
            make_oauth_account("acct_pro", "pro"),
            make_oauth_account("acct_free", "free"),
            make_oauth_account("acct_team", "team"),
        ];
        svc.sanitize_collection(&mut collection, &valid);

        assert_eq!(collection.accounts.len(), 2);
        assert!(collection.accounts.contains(&"acct_pro".to_string()));
        assert!(collection.accounts.contains(&"acct_team".to_string()));
        assert!(!collection.accounts.contains(&"acct_free".to_string()));
    }

    #[test]
    fn test_generate_local_api_key() {
        let key = LocalAccessService::generate_local_api_key();
        assert!(key.starts_with("sk-local-"));
        assert_eq!(key.len(), 9 + 32);
    }

    #[test]
    fn test_record_request() {
        let tmp = TempDir::new().unwrap();
        let (svc, _collection) = setup(&tmp);

        let event = LocalAccessStatsEvent {
            timestamp: Utc::now().to_rfc3339(),
            account_id: "acct_oauth_pro_001".to_string(),
            model: "gpt-4o".to_string(),
            status: 200,
            latency_ms: 1200,
            tokens_in: 150,
            tokens_out: 400,
            is_stream: false,
        };

        svc.record_request(event).unwrap();

        let stats: LocalAccessStatsFile =
            read_json_file_opt(&svc.paths.codex_local_access_stats_file)
                .unwrap()
                .unwrap();
        assert_eq!(stats.requests.len(), 1);
        assert_eq!(stats.requests[0].model, "gpt-4o");
    }

    #[test]
    fn test_record_request_trims_excess() {
        let tmp = TempDir::new().unwrap();
        let (svc, _collection) = setup(&tmp);

        for i in 0..1050 {
            let event = LocalAccessStatsEvent {
                timestamp: Utc::now().to_rfc3339(),
                account_id: format!("acct_{}", i),
                model: "gpt-4o".to_string(),
                status: 200,
                latency_ms: 100,
                tokens_in: 10,
                tokens_out: 20,
                is_stream: false,
            };
            svc.record_request(event).unwrap();
        }

        let stats: LocalAccessStatsFile =
            read_json_file_opt(&svc.paths.codex_local_access_stats_file)
                .unwrap()
                .unwrap();
        assert_eq!(stats.requests.len(), 1000);
    }

    #[test]
    fn test_clear_stats() {
        let tmp = TempDir::new().unwrap();
        let (svc, _collection) = setup(&tmp);

        let event = LocalAccessStatsEvent {
            timestamp: Utc::now().to_rfc3339(),
            account_id: "acct_test".to_string(),
            model: "gpt-4o".to_string(),
            status: 200,
            latency_ms: 100,
            tokens_in: 10,
            tokens_out: 20,
            is_stream: false,
        };
        svc.record_request(event).unwrap();
        assert!(svc.paths.codex_local_access_stats_file.exists());

        svc.clear_stats().unwrap();
        assert!(!svc.paths.codex_local_access_stats_file.exists());
    }

    #[test]
    fn test_set_enabled() {
        let tmp = TempDir::new().unwrap();
        let (svc, mut collection) = setup(&tmp);

        svc.set_enabled(&mut collection, true).unwrap();
        assert!(collection.enabled);

        let loaded = svc.load_collection().unwrap();
        assert!(loaded.enabled);
    }

    #[test]
    fn test_update_port() {
        let tmp = TempDir::new().unwrap();
        let (svc, mut collection) = setup(&tmp);

        svc.update_port(&mut collection, 9999).unwrap();
        assert_eq!(collection.port, 9999);

        let loaded = svc.load_collection().unwrap();
        assert_eq!(loaded.port, 9999);
    }
}
