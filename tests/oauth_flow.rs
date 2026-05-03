use oauthcodex::adapters::events::EventEmitter;
use oauthcodex::adapters::fs_store::CodexPaths;
use oauthcodex::domain::codex_models::*;
use oauthcodex::domain::oauth::{
    generate_pkce_verifier, parse_query_pairs, OAuthConfig, OAuthEvent, OAuthPending, OAuthService,
};
use std::sync::Arc;
use tempfile::TempDir;

fn setup() -> (OAuthService, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let paths = CodexPaths::for_tests(tmp.path());
    let svc = OAuthService::new(paths);
    (svc, tmp)
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("bind free port");
    listener.local_addr().expect("addr").port()
}

#[test]
fn test_full_oauth_flow_start_to_cancel() {
    let (svc, _tmp) = setup();
    let (auth_url, _verifier, pending) = svc.start_oauth_login(free_port()).expect("start");
    assert!(auth_url.contains("client_id="));
    assert!(auth_url.contains("state="));
    assert!(auth_url.contains("code_challenge="));
    assert!(!pending.state.is_empty());

    svc.cancel_login(&pending.login_id).expect("cancel");
    let remaining = svc.load_pending().expect("load");
    assert!(remaining.is_none());
}

#[test]
fn test_manual_callback_parsing_variants() {
    let (svc, _tmp) = setup();

    let full_url = "http://localhost:1455/auth/callback?code=abc&state=def";
    let (code1, state1) = svc.parse_manual_callback(full_url).expect("full");
    assert_eq!(code1, "abc");
    assert_eq!(state1, "def");

    let path_url = "/auth/callback?code=ghi&state=jkl";
    let (code2, state2) = svc.parse_manual_callback(path_url).expect("path");
    assert_eq!(code2, "ghi");
    assert_eq!(state2, "jkl");

    let raw_query = "code=mno&state=pqr";
    let (code3, state3) = svc.parse_manual_callback(raw_query).expect("raw");
    assert_eq!(code3, "mno");
    assert_eq!(state3, "pqr");
}

#[test]
fn test_manual_callback_decodes_raw_query() {
    let (svc, _tmp) = setup();

    let (code, state) = svc
        .parse_manual_callback("code=abc%2F123&state=state%20with%20space")
        .expect("raw encoded");
    assert_eq!(code, "abc/123");
    assert_eq!(state, "state with space");
}

#[test]
fn test_start_with_expired_pending_creates_new() {
    let (svc, _tmp) = setup();
    let port = free_port();
    let (_, _, pending1) = svc.start_oauth_login(port).expect("start");
    let old_state = pending1.state.clone();

    let stale = OAuthPending {
        login_id: pending1.login_id.clone(),
        state: pending1.state.clone(),
        code_verifier: pending1.code_verifier.clone(),
        code_challenge: pending1.code_challenge.clone(),
        created_at: "2020-01-01T00:00:00Z".into(),
        expires_at: "2020-01-01T00:05:00Z".into(),
    };
    svc.save_pending(&stale).expect("save stale");

    let (_, _, pending2) = svc.start_oauth_login(port).expect("restart");
    assert_ne!(pending2.state, old_state);
}

#[test]
fn test_state_mismatch_on_complete() {
    let (svc, _tmp) = setup();
    let pending = OAuthPending::new();
    let pairs = vec![
        ("code".to_string(), "good_code".to_string()),
        ("state".to_string(), "bad_state".to_string()),
    ];
    let result = svc.complete_oauth_login(&pairs, &pending);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("State mismatch"));
}

#[test]
fn test_valid_complete_extracts_code() {
    let (svc, _tmp) = setup();
    let pending = OAuthPending::new();
    let pairs = vec![
        ("code".to_string(), "valid_code_123".to_string()),
        ("state".to_string(), pending.state.clone()),
    ];
    let (code, cv) = svc
        .complete_oauth_login(&pairs, &pending)
        .expect("complete");
    assert_eq!(code, "valid_code_123");
    assert_eq!(cv, pending.code_verifier);
}

#[test]
fn test_complete_rejects_stale_login_id() {
    let (svc, _tmp) = setup();
    let (_, _, pending) = svc.start_oauth_login(free_port()).expect("start");
    let pairs = vec![
        ("code".to_string(), "valid_code_123".to_string()),
        ("state".to_string(), pending.state.clone()),
    ];

    let result = svc.complete_oauth_login_for_login("wrong_login_id", &pairs);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Stale login id"));
}

#[test]
fn test_auth_url_constants_match_fixture() {
    let config = OAuthConfig::default();
    assert_eq!(config.client_id, "U2BcmqLqFwsFpMk8T5r5GqVqVBFx5RkP");
    assert_eq!(
        config.auth_endpoint,
        "https://auth.openai.com/oauth/authorize"
    );
    assert_eq!(config.token_endpoint, "https://auth.openai.com/oauth/token");
    assert_eq!(config.redirect_uri, "http://localhost:1455/auth/callback");
    assert_eq!(
        config.scopes,
        "openid profile email offline_access api.connectors.read api.connectors.invoke"
    );
    assert_eq!(config.originator, "codex_vscode");
    assert_eq!(config.timeout_secs, 300);
    assert_eq!(config.callback_port, 1455);
}

#[test]
fn test_pkce_verifier_and_challenge_deterministic() {
    let verifier = "fixed_test_verifier_abcdefghijklmnopqrstuvwxyz";
    let _ = generate_pkce_verifier(); // just testing that generate works
    let challenge2 = oauthcodex::domain::oauth::generate_code_challenge(verifier);
    // Challenge from verifier should be reproducible
    let challenge3 = oauthcodex::domain::oauth::generate_code_challenge(verifier);
    assert_eq!(challenge2, challenge3);
}

#[test]
fn test_event_emitter_oauth_events() {
    let emitter = Arc::new(EventEmitter::new());
    let completed = std::sync::Arc::new(std::sync::Mutex::new(false));
    let c = completed.clone();
    emitter.on("codex-oauth-login-completed", move |_| {
        *c.lock().unwrap() = true;
    });
    emitter.emit(OAuthEvent::LoginCompleted {
        login_id: "test".into(),
        account: Box::new(CodexAccount {
            id: "test".into(),
            provider: "codex".into(),
            auth_mode: CodexAuthMode::OAuth,
            email: None,
            plan_type: None,
            account_id: None,
            organization_id: None,
            organizations: vec![],
            display_name: "test".into(),
            tags: vec![],
            tokens: CodexTokens::empty(),
            api_key: None,
            base_url: None,
            provider_id: None,
            provider_name: None,
            api_provider_mode: None,
            quota: None,
            created_at: None,
            last_used: None,
            last_refresh: None,
        }),
    });
    assert!(*completed.lock().unwrap());
}

#[test]
fn test_timeout_event() {
    let emitter = Arc::new(EventEmitter::new());
    let timed_out = std::sync::Arc::new(std::sync::Mutex::new(false));
    let t = timed_out.clone();
    emitter.on("codex-oauth-login-timeout", move |_| {
        *t.lock().unwrap() = true;
    });
    emitter.emit(OAuthEvent::LoginTimeout {
        login_id: "timeout_test".into(),
    });
    assert!(*timed_out.lock().unwrap());
}

#[test]
fn test_port_available_check() {
    let (svc, _tmp) = setup();
    let available = svc.check_port_available(19999).expect("check");
    assert!(available);
}

#[test]
fn test_start_oauth_fails_when_callback_port_busy() {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("bind test port");
    let busy_port = listener.local_addr().expect("addr").port();
    let (svc, _tmp) = setup();

    let result = svc.start_oauth_login(busy_port);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already in use"));
}

#[test]
fn test_parse_query_pairs() {
    let pairs = parse_query_pairs("a=1&b=2&empty=");
    assert_eq!(pairs.len(), 3);
    assert_eq!(pairs[0], ("a".to_string(), "1".to_string()));
    assert_eq!(pairs[1], ("b".to_string(), "2".to_string()));
    assert_eq!(pairs[2], ("empty".to_string(), "".to_string()));
}

#[test]
fn test_decode_jwt_payload() {
    let token = "eyJhbGciOiJSUzI1NiJ9.eyJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJleHAiOjk5OTk5OTk5OTksImh0dHBzOi8vYXBpLm9wZW5haS5jb20vYXV0aCI6eyJhY2NvdW50X2lkIjoiYWNjdF90ZXN0IiwicGxhbl90eXBlIjoicHJvIiwib3JnYW5pemF0aW9ucyI6WyJkZXZlbG9wZXIiXX19.fake";
    let (payload, tokens) = oauthcodex::domain::oauth::decode_jwt_payload(token).expect("decode");
    assert_eq!(payload.email, Some("test@example.com".into()));
    assert!(payload.auth.is_some());
    assert_eq!(payload.auth.as_ref().unwrap().account_id, "acct_test");
    assert!(tokens.is_some());
}

#[test]
fn test_malformed_jwt() {
    let result = oauthcodex::domain::oauth::decode_jwt_payload("not.a.jwt");
    assert!(result.is_err());
}

#[test]
fn test_extract_account_from_tokens() {
    let token = "eyJhbGciOiJSUzI1NiJ9.eyJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJleHAiOjk5OTk5OTk5OTksImh0dHBzOi8vYXBpLm9wZW5haS5jb20vYXV0aCI6eyJhY2NvdW50X2lkIjoiYWNjdF90ZXN0IiwicGxhbl90eXBlIjoicHJvIiwib3JnYW5pemF0aW9ucyI6WyJkZXZlbG9wZXIiXX19.fake";
    let (payload, _) = oauthcodex::domain::oauth::decode_jwt_payload(token).expect("decode");
    let tokens = CodexTokens {
        access_token: Some("at_test".into()),
        refresh_token: Some("rt_test".into()),
        id_token: Some(token.into()),
        token_type: Some("Bearer".into()),
        expires_at: Some("9999-12-31T00:00:00Z".into()),
        scope: Some("openid".into()),
    };
    let account = oauthcodex::domain::oauth::extract_account_from_tokens(&tokens, Some(&payload))
        .expect("extract");
    assert!(account.is_oauth());
    assert_eq!(account.email, Some("test@example.com".into()));
    assert_eq!(account.plan_type, Some("pro".into()));
}

#[test]
fn test_cancel_current() {
    let (svc, _tmp) = setup();
    let (_, _, _) = svc.start_oauth_login(free_port()).expect("start");
    assert!(svc.has_active_pending().expect("check"));
    svc.cancel_current().expect("cancel");
    assert!(!svc.has_active_pending().expect("check"));
}
