use crate::adapters::fs_store::{read_json_file_opt, write_json_atomic, CodexPaths};
use crate::domain::codex_models::*;
use crate::error::CodexError;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use chrono::Utc;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const AUTH_ENDPOINT: &str = "https://auth.openai.com/oauth/authorize";
const TOKEN_ENDPOINT: &str = "https://auth.openai.com/oauth/token";
const REDIRECT_URI: &str = "http://localhost:1455/auth/callback";
const SCOPES: &str =
    "openid profile email offline_access api.connectors.read api.connectors.invoke";
const ORIGINATOR: &str = "codex_vscode";
const OAUTH_TIMEOUT_SECS: u64 = 300;
const CALLBACK_PORT: u16 = 1455;

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub auth_endpoint: String,
    pub token_endpoint: String,
    pub redirect_uri: String,
    pub scopes: String,
    pub originator: String,
    pub timeout_secs: u64,
    pub callback_port: u16,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            client_id: CLIENT_ID.to_string(),
            auth_endpoint: AUTH_ENDPOINT.to_string(),
            token_endpoint: TOKEN_ENDPOINT.to_string(),
            redirect_uri: REDIRECT_URI.to_string(),
            scopes: SCOPES.to_string(),
            originator: ORIGINATOR.to_string(),
            timeout_secs: OAUTH_TIMEOUT_SECS,
            callback_port: CALLBACK_PORT,
        }
    }
}

fn random_hex(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| format!("{:x}", rng.gen_range(0..16)))
        .collect()
}

pub fn generate_pkce_verifier() -> String {
    let mut bytes = vec![0u8; 32];
    rand::thread_rng().fill(&mut bytes[..]);
    URL_SAFE_NO_PAD.encode(&bytes)
}

pub fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let digest = hasher.finalize();
    URL_SAFE_NO_PAD.encode(digest)
}

pub fn generate_login_id() -> String {
    format!("login_{}", random_hex(16))
}

pub fn generate_state() -> String {
    format!("st_{}", random_hex(32))
}

#[derive(Debug, Clone, PartialEq)]
pub struct OAuthPending {
    pub login_id: String,
    pub state: String,
    pub code_verifier: String,
    pub code_challenge: String,
    pub redirect_uri: String,
    pub port: u16,
    pub code: Option<String>,
    pub created_at: String,
    pub expires_at: String,
}

impl OAuthPending {
    pub fn new() -> Self {
        Self::new_for_port(CALLBACK_PORT)
    }

    pub fn new_for_port(port: u16) -> Self {
        let verifier = generate_pkce_verifier();
        let challenge = generate_code_challenge(&verifier);
        let now = Utc::now();
        let redirect_uri = callback_redirect_uri(port);
        Self {
            login_id: generate_login_id(),
            state: generate_state(),
            code_verifier: verifier,
            code_challenge: challenge,
            redirect_uri,
            port,
            code: None,
            created_at: now.to_rfc3339(),
            expires_at: (now + chrono::Duration::seconds(OAUTH_TIMEOUT_SECS as i64)).to_rfc3339(),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(&self.expires_at) {
            Utc::now() > exp
        } else {
            true
        }
    }
}

fn callback_redirect_uri(port: u16) -> String {
    format!("http://localhost:{port}/auth/callback")
}

impl Default for OAuthPending {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_auth_url(config: &OAuthConfig, pending: &OAuthPending) -> String {
    format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256&originator={}&id_token_add_organizations=true&codex_cli_simplified_flow=true",
        config.auth_endpoint,
        url_encode(&config.client_id),
        url_encode(&config.redirect_uri),
        url_encode(&config.scopes),
        url_encode(&pending.state),
        url_encode(&pending.code_challenge),
        url_encode(&config.originator),
    )
}

fn url_encode(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                result.push(byte as char)
            }
            b' ' => result.push_str("%20"),
            _ => result.push_str(&format!("%{:02X}", byte)),
        }
    }
    result
}

#[derive(Debug, Clone)]
pub enum OAuthEvent {
    LoginCompleted { login_id: String },
    LoginTimeout { login_id: String },
    LoginCancelled { login_id: String },
    LoginError { login_id: String, error: String },
}

impl OAuthEvent {
    pub fn event_name(&self) -> &str {
        match self {
            OAuthEvent::LoginCompleted { .. } => "codex-oauth-login-completed",
            OAuthEvent::LoginTimeout { .. } => "codex-oauth-login-timeout",
            OAuthEvent::LoginCancelled { .. } => "codex-oauth-login-cancelled",
            OAuthEvent::LoginError { .. } => "codex-oauth-login-error",
        }
    }
}

pub struct OAuthService {
    config: OAuthConfig,
    paths: CodexPaths,
    pending_lock: Arc<Mutex<Option<OAuthPending>>>,
}

impl OAuthService {
    pub fn new(paths: CodexPaths) -> Self {
        Self::with_config(paths, OAuthConfig::default())
    }

    pub fn with_config(paths: CodexPaths, config: OAuthConfig) -> Self {
        Self {
            config,
            paths,
            pending_lock: Arc::new(Mutex::new(None)),
        }
    }

    pub fn config(&self) -> &OAuthConfig {
        &self.config
    }

    pub fn start_oauth_login(
        &self,
        port: u16,
    ) -> Result<(String, String, OAuthPending), CodexError> {
        let callback_port = if port == 0 {
            self.config.callback_port
        } else {
            port
        };
        if !self.check_port_available(callback_port)? {
            return Err(CodexError::OAuth(format!(
                "OAuth callback port {callback_port} is already in use"
            )));
        }

        let existing = self.load_pending()?;
        let pending: OAuthPending = if let Some(ref existing_pending) = existing {
            if !existing_pending.is_expired() && existing_pending.port == callback_port {
                existing_pending.clone()
            } else {
                let new_pending = OAuthPending::new_for_port(callback_port);
                self.save_pending(&new_pending)?;
                new_pending
            }
        } else {
            let new_pending = OAuthPending::new_for_port(callback_port);
            self.save_pending(&new_pending)?;
            new_pending
        };

        let request_config = OAuthConfig {
            redirect_uri: pending.redirect_uri.clone(),
            callback_port,
            ..self.config.clone()
        };
        let auth_url = build_auth_url(&request_config, &pending);
        let code_verifier = pending.code_verifier.clone();

        {
            let mut guard = self.pending_lock.lock().unwrap();
            *guard = Some(pending.clone());
        }

        Ok((auth_url, code_verifier, pending))
    }

    pub fn parse_manual_callback(&self, input: &str) -> Result<(String, String), CodexError> {
        let pairs = if input.starts_with("http://") || input.starts_with("https://") {
            let url = url::Url::parse(input)?;
            url.query_pairs()
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect::<Vec<_>>()
        } else if input.starts_with("/auth/callback?") {
            let stripped = input.strip_prefix("/auth/callback?").unwrap();
            parse_query_pairs(stripped)
        } else {
            parse_query_pairs(input)
        };

        let code = pairs
            .iter()
            .find(|(k, _)| k == "code")
            .map(|(_, v)| v.clone())
            .ok_or_else(|| CodexError::OAuth("Missing 'code' parameter".into()))?;
        let state = pairs
            .iter()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.clone())
            .ok_or_else(|| CodexError::OAuth("Missing 'state' parameter".into()))?;

        if code.is_empty() {
            return Err(CodexError::OAuth("Code parameter is empty".into()));
        }
        if state.is_empty() {
            return Err(CodexError::OAuth("State parameter is empty".into()));
        }

        Ok((code, state))
    }

    pub fn complete_oauth_login(
        &self,
        callback_pairs: &[(String, String)],
        pending: &OAuthPending,
    ) -> Result<(String, String), CodexError> {
        let code = callback_pairs
            .iter()
            .find(|(k, _)| k == "code")
            .map(|(_, v)| v.clone())
            .ok_or_else(|| CodexError::OAuth("Missing authorization code".into()))?;
        let state = callback_pairs
            .iter()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.clone())
            .ok_or_else(|| CodexError::OAuth("Missing state in callback".into()))?;

        if state != pending.state {
            return Err(CodexError::OAuth(format!(
                "State mismatch: expected {} got {}",
                pending.state, state
            )));
        }

        if code.is_empty() {
            return Err(CodexError::OAuth("Authorization code is empty".into()));
        }

        Ok((code, pending.code_verifier.clone()))
    }

    pub fn complete_oauth_login_for_login(
        &self,
        login_id: &str,
        callback_pairs: &[(String, String)],
    ) -> Result<(String, String), CodexError> {
        let pending = self
            .load_pending()?
            .ok_or_else(|| CodexError::OAuth("No active OAuth login".into()))?;

        if pending.is_expired() {
            return Err(CodexError::AuthState(format!(
                "OAuth login expired: {}",
                pending.login_id
            )));
        }

        if pending.login_id != login_id {
            return Err(CodexError::AuthState(format!(
                "Stale login id: expected {} got {}",
                pending.login_id, login_id
            )));
        }

        let result = self.complete_oauth_login(callback_pairs, &pending);
        if result.is_ok() {
            self.clear_pending()?;
        }
        result
    }

    pub fn cancel_login(&self, login_id: &str) -> Result<(), CodexError> {
        let pending = self.load_pending()?;
        match pending {
            Some(p) if p.login_id == login_id => {
                self.clear_pending()?;
                Ok(())
            }
            _ => Err(CodexError::OAuth(format!(
                "No active login with id: {}",
                login_id
            ))),
        }
    }

    pub fn cancel_current(&self) -> Result<(), CodexError> {
        self.clear_pending()
    }

    pub fn save_pending(&self, pending: &OAuthPending) -> Result<(), CodexError> {
        let state = CodexPendingOAuthState {
            login_id: pending.login_id.clone(),
            state: pending.state.clone(),
            code_verifier: pending.code_verifier.clone(),
            code_challenge: pending.code_challenge.clone(),
            redirect_uri: Some(pending.redirect_uri.clone()),
            port: Some(pending.port),
            code: pending.code.clone(),
            created_at: Some(pending.created_at.clone()),
            expires_at: Some(pending.expires_at.clone()),
        };
        write_json_atomic(&self.paths.codex_oauth_pending_file, &state)
    }

    pub fn load_pending(&self) -> Result<Option<OAuthPending>, CodexError> {
        let state: Option<CodexPendingOAuthState> =
            read_json_file_opt(&self.paths.codex_oauth_pending_file)?;
        Ok(state.map(|s| OAuthPending {
            login_id: s.login_id,
            state: s.state,
            code_verifier: s.code_verifier,
            code_challenge: s.code_challenge,
            redirect_uri: s
                .redirect_uri
                .unwrap_or_else(|| callback_redirect_uri(s.port.unwrap_or(CALLBACK_PORT))),
            port: s.port.unwrap_or(CALLBACK_PORT),
            code: s.code,
            created_at: s.created_at.unwrap_or_default(),
            expires_at: s.expires_at.unwrap_or_default(),
        }))
    }

    pub fn clear_pending(&self) -> Result<(), CodexError> {
        let file = &self.paths.codex_oauth_pending_file;
        if file.exists() {
            std::fs::remove_file(file).map_err(CodexError::Io)?;
        }
        let mut guard = self.pending_lock.lock().unwrap();
        *guard = None;
        Ok(())
    }

    pub fn has_active_pending(&self) -> Result<bool, CodexError> {
        let pending = self.load_pending()?;
        Ok(pending.is_some_and(|p| !p.is_expired()))
    }

    pub fn check_port_available(&self, port: u16) -> Result<bool, CodexError> {
        match std::net::TcpListener::bind(("127.0.0.1", port)) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => Ok(false),
            Err(e) => Err(CodexError::Io(e)),
        }
    }
}

pub fn parse_query_pairs(query: &str) -> Vec<(String, String)> {
    url::form_urlencoded::parse(query.as_bytes())
        .filter_map(|(key, value)| {
            if key.is_empty() {
                None
            } else {
                Some((key.into_owned(), value.into_owned()))
            }
        })
        .collect()
}

pub fn decode_jwt_payload(id_token: &str) -> Result<(JwtPayload, Option<CodexTokens>), CodexError> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() < 2 {
        return Err(CodexError::Jwt("Malformed JWT: missing parts".into()));
    }
    let payload_b64 = parts[1];
    let payload_json = URL_SAFE_NO_PAD
        .decode(payload_b64.as_bytes())
        .map_err(|e| CodexError::Jwt(format!("Base64 decode error: {e}")))?;
    let payload: JwtPayload = serde_json::from_slice(&payload_json)
        .map_err(|e| CodexError::Jwt(format!("JSON parse error: {e}")))?;

    let tokens = if parts.len() >= 3 {
        Some(CodexTokens {
            access_token: None,
            refresh_token: None,
            id_token: Some(id_token.to_string()),
            token_type: Some("Bearer".into()),
            expires_at: payload.exp.map(|e| {
                chrono::DateTime::from_timestamp(e as i64, 0)
                    .unwrap_or_default()
                    .to_rfc3339()
            }),
            scope: Some(SCOPES.to_string()),
        })
    } else {
        None
    };

    Ok((payload, tokens))
}

pub fn extract_account_from_tokens(
    tokens: &CodexTokens,
    payload: Option<&JwtPayload>,
) -> Result<CodexAccount, CodexError> {
    let auth = payload
        .and_then(|p| p.auth.as_ref())
        .ok_or_else(|| CodexError::Token("Missing auth claim in JWT".into()))?;

    let email = payload.and_then(|p| p.email.clone());
    let sub = payload.and_then(|p| p.sub.clone());

    let id = format!("acct_oauth_{}", auth.account_id);
    let display_name = email
        .clone()
        .unwrap_or_else(|| sub.clone().unwrap_or_default());

    Ok(CodexAccount {
        id,
        provider: "codex".into(),
        auth_mode: CodexAuthMode::OAuth,
        email,
        plan_type: Some(auth.plan_type.clone()),
        account_id: Some(auth.account_id.clone()),
        organization_id: auth.organization_id.clone(),
        organizations: auth.organizations.clone(),
        display_name,
        tags: vec![],
        tokens: tokens.clone(),
        api_key: None,
        base_url: None,
        provider_id: None,
        provider_name: None,
        api_provider_mode: None,
        quota: None,
        created_at: Some(Utc::now().to_rfc3339()),
        last_used: None,
        last_refresh: Some(Utc::now().to_rfc3339()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn free_port() -> u16 {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("bind free port");
        listener.local_addr().expect("addr").port()
    }

    #[test]
    fn test_generate_pkce_verifier_length() {
        let verifier = generate_pkce_verifier();
        assert!(!verifier.is_empty());
        assert!(verifier.len() >= 43);
    }

    #[test]
    fn test_code_challenge_deterministic() {
        let verifier = "test_verifier_string_for_pkce";
        let challenge1 = generate_code_challenge(verifier);
        let challenge2 = generate_code_challenge(verifier);
        assert_eq!(challenge1, challenge2);
        assert!(!challenge1.is_empty());
    }

    #[test]
    fn test_build_auth_url() {
        let config = OAuthConfig::default();
        let pending = OAuthPending::new();
        let url = build_auth_url(&config, &pending);
        assert!(url.starts_with("https://auth.openai.com/oauth/authorize"));
        assert!(url.contains("client_id="));
        assert!(url.contains("code_challenge="));
        assert!(url.contains("state="));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("originator=codex_vscode"));
        assert!(url.contains("id_token_add_organizations=true"));
        assert!(url.contains("codex_cli_simplified_flow=true"));
    }

    #[test]
    fn test_parse_manual_callback_full_url() {
        let input = "http://localhost:1455/auth/callback?code=test_code_123&state=test_state_456";
        let svc = OAuthService::new(CodexPaths::for_tests(std::path::Path::new("/tmp")));
        let result = svc.parse_manual_callback(input).expect("parse");
        assert_eq!(result.0, "test_code_123");
        assert_eq!(result.1, "test_state_456");
    }

    #[test]
    fn test_parse_manual_callback_path() {
        let svc = OAuthService::new(CodexPaths::for_tests(std::path::Path::new("/tmp")));
        let result = svc
            .parse_manual_callback("/auth/callback?code=abc&state=def")
            .expect("parse");
        assert_eq!(result.0, "abc");
        assert_eq!(result.1, "def");
    }

    #[test]
    fn test_parse_manual_callback_query() {
        let svc = OAuthService::new(CodexPaths::for_tests(std::path::Path::new("/tmp")));
        let result = svc
            .parse_manual_callback("code=abc123&state=def456")
            .expect("parse");
        assert_eq!(result.0, "abc123");
        assert_eq!(result.1, "def456");
    }

    #[test]
    fn test_parse_missing_code() {
        let svc = OAuthService::new(CodexPaths::for_tests(std::path::Path::new("/tmp")));
        let result = svc.parse_manual_callback("state=def456");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing 'code'"));
    }

    #[test]
    fn test_pending_expiration() {
        let mut pending = OAuthPending::new();
        assert!(!pending.is_expired());
        pending.expires_at = "2020-01-01T00:00:00Z".to_string();
        assert!(pending.is_expired());
    }

    #[test]
    fn test_save_and_load_pending() {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let paths = CodexPaths::for_tests(tmp.path());
        let svc = OAuthService::new(paths);

        let pending = OAuthPending::new();
        svc.save_pending(&pending).expect("save");
        let loaded = svc.load_pending().expect("load");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().state, pending.state);
    }

    #[test]
    fn test_clear_pending() {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let paths = CodexPaths::for_tests(tmp.path());
        let svc = OAuthService::new(paths);

        let pending = OAuthPending::new();
        svc.save_pending(&pending).expect("save");
        svc.clear_pending().expect("clear");
        let loaded = svc.load_pending().expect("load");
        assert!(loaded.is_none());
    }

    #[test]
    fn test_state_mismatch() {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let paths = CodexPaths::for_tests(tmp.path());
        let svc = OAuthService::new(paths);

        let pending = OAuthPending::new();
        let pairs = vec![
            ("code".to_string(), "test_code".to_string()),
            ("state".to_string(), "wrong_state".to_string()),
        ];
        let result = svc.complete_oauth_login(&pairs, &pending);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("State mismatch"));
    }

    #[test]
    fn test_start_oauth_creates_pending() {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let paths = CodexPaths::for_tests(tmp.path());
        let svc = OAuthService::new(paths);

        let (auth_url, _, pending) = svc.start_oauth_login(free_port()).expect("start");
        assert!(auth_url.starts_with("https://"));
        assert!(!pending.state.is_empty());
        assert!(!pending.login_id.is_empty());
    }

    #[test]
    fn test_cancel_by_login_id() {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let paths = CodexPaths::for_tests(tmp.path());
        let svc = OAuthService::new(paths);

        let (_, _, pending) = svc.start_oauth_login(free_port()).expect("start");
        svc.cancel_login(&pending.login_id).expect("cancel");
        let loaded = svc.load_pending().expect("load");
        assert!(loaded.is_none());
    }

    #[test]
    fn test_cancel_wrong_id() {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let paths = CodexPaths::for_tests(tmp.path());
        let svc = OAuthService::new(paths);

        svc.start_oauth_login(free_port()).expect("start");
        let result = svc.cancel_login("wrong_id");
        assert!(result.is_err());
    }
}
