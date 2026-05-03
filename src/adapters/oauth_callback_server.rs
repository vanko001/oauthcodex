use crate::adapters::events::EventEmitter;
use crate::domain::oauth::OAuthEvent;
use crate::error::CodexError;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::sync::oneshot;

#[derive(Debug, Clone)]
pub struct CallbackServerState {
    pub login_id: String,
    pub expected_state: String,
    pub code_verifier: String,
    pub emitter: Arc<EventEmitter>,
    pub pending_file_path: std::path::PathBuf,
    pub timeout_sender: Arc<StdMutex<Option<oneshot::Sender<()>>>>,
    pub result_sender: Arc<StdMutex<Option<oneshot::Sender<CallbackResult>>>>,
}

#[derive(Debug, Clone)]
pub enum CallbackResult {
    Completed { code: String, state: String },
    Cancelled,
    Timeout,
}

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
}

async fn handle_callback(
    State(state): State<Arc<CallbackServerState>>,
    Query(params): Query<CallbackParams>,
) -> impl IntoResponse {
    match &params.code {
        Some(code) if !code.is_empty() => {
            let state_match = params.state.as_deref() == Some(&state.expected_state);
            if state_match {
                if let Ok(mut guard) = state.result_sender.lock() {
                    if let Some(sender) = guard.take() {
                        let _ = sender.send(CallbackResult::Completed {
                            code: code.clone(),
                            state: state.expected_state.clone(),
                        });
                    }
                }
                state.emitter.emit(OAuthEvent::LoginCompleted {
                    login_id: state.login_id.clone(),
                    account: Box::new(crate::domain::codex_models::CodexAccount {
                        id: format!("pending_{}", state.login_id),
                        provider: "codex".into(),
                        auth_mode: crate::domain::codex_models::CodexAuthMode::OAuth,
                        email: None,
                        plan_type: None,
                        account_id: None,
                        organization_id: None,
                        organizations: vec![],
                        display_name: "".into(),
                        tags: vec![],
                        tokens: crate::domain::codex_models::CodexTokens::empty(),
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
                Html(SUCCESS_HTML.to_string())
            } else {
                Html(ERROR_HTML_STATE_MISMATCH.to_string())
            }
        }
        _ => Html(ERROR_HTML_NO_CODE.to_string()),
    }
}

async fn handle_cancel(State(state): State<Arc<CallbackServerState>>) -> impl IntoResponse {
    if let Ok(mut guard) = state.result_sender.lock() {
        if let Some(sender) = guard.take() {
            let _ = sender.send(CallbackResult::Cancelled);
        }
    }
    state.emitter.emit(OAuthEvent::LoginCancelled {
        login_id: state.login_id.clone(),
    });
    Html("<h1>Login cancelled</h1><p>You can close this window.</p>".to_string())
}

const SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>Login Successful</title></head>
<body>
<h1>Login Successful</h1>
<p>You can close this window. The application will complete the login process.</p>
</body>
</html>"#;

const ERROR_HTML_NO_CODE: &str = r#"<!DOCTYPE html>
<html>
<head><title>Login Error</title></head>
<body>
<h1>Login Error</h1>
<p>No authorization code received. Please try again.</p>
</body>
</html>"#;

const ERROR_HTML_STATE_MISMATCH: &str = r#"<!DOCTYPE html>
<html>
<head><title>Login Error</title></head>
<body>
<h1>Login Error</h1>
<p>State mismatch. The login attempt may have been tampered with. Please try again.</p>
</body>
</html>"#;

pub fn build_callback_app(state: Arc<CallbackServerState>) -> Router {
    Router::new()
        .route("/auth/callback", get(handle_callback))
        .route("/cancel", get(handle_cancel))
        .with_state(state)
}

pub fn start_callback_server(
    port: u16,
    login_id: String,
    expected_state: String,
    code_verifier: String,
    emitter: Arc<EventEmitter>,
    pending_file_path: std::path::PathBuf,
    _timeout_secs: u64,
) -> Result<oneshot::Receiver<CallbackResult>, CodexError> {
    let (result_sender, result_receiver) = oneshot::channel();
    let (_timeout_sender, _timeout_receiver) = oneshot::channel();

    let state = Arc::new(CallbackServerState {
        login_id,
        expected_state,
        code_verifier,
        emitter,
        pending_file_path,
        timeout_sender: Arc::new(StdMutex::new(Some(_timeout_sender))),
        result_sender: Arc::new(StdMutex::new(Some(result_sender))),
    });

    let app = build_callback_app(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = std::net::TcpListener::bind(addr).map_err(|e| {
        CodexError::OAuth(format!(
            "Failed to bind callback server on port {port}: {e}"
        ))
    })?;
    listener.set_nonblocking(true).map_err(CodexError::Io)?;

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            axum::serve(listener, app).await.unwrap();
        });
    });

    Ok(result_receiver)
}
