use crate::adapters::events::EventEmitter;
use crate::adapters::fs_store::{read_json_file_opt, write_json_atomic};
use crate::domain::codex_models::CodexPendingOAuthState;
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
    pub shutdown_sender: Arc<StdMutex<Option<oneshot::Sender<()>>>>,
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
    if let Some(error) = params.error.as_deref() {
        let message = params
            .error_description
            .clone()
            .unwrap_or_else(|| error.to_string());
        send_result_once(&state, CallbackResult::Cancelled);
        state.emitter.emit(OAuthEvent::LoginError {
            login_id: state.login_id.clone(),
            error: message,
        });
        request_shutdown(&state);
        return Html(ERROR_HTML_NO_CODE.to_string());
    }

    match &params.code {
        Some(code) if !code.is_empty() => {
            let state_match = params.state.as_deref() == Some(&state.expected_state);
            if state_match {
                let _ = persist_callback_code(&state, code);
                send_result_once(
                    &state,
                    CallbackResult::Completed {
                        code: code.clone(),
                        state: state.expected_state.clone(),
                    },
                );
                state.emitter.emit(OAuthEvent::LoginCompleted {
                    login_id: state.login_id.clone(),
                });
                request_shutdown(&state);
                Html(SUCCESS_HTML.to_string())
            } else {
                state.emitter.emit(OAuthEvent::LoginError {
                    login_id: state.login_id.clone(),
                    error: "State mismatch".into(),
                });
                Html(ERROR_HTML_STATE_MISMATCH.to_string())
            }
        }
        _ => Html(ERROR_HTML_NO_CODE.to_string()),
    }
}

async fn handle_cancel(State(state): State<Arc<CallbackServerState>>) -> impl IntoResponse {
    send_result_once(&state, CallbackResult::Cancelled);
    state.emitter.emit(OAuthEvent::LoginCancelled {
        login_id: state.login_id.clone(),
    });
    request_shutdown(&state);
    Html("<h1>Login cancelled</h1><p>You can close this window.</p>".to_string())
}

fn send_result_once(state: &CallbackServerState, result: CallbackResult) {
    if let Ok(mut guard) = state.result_sender.lock() {
        if let Some(sender) = guard.take() {
            let _ = sender.send(result);
        }
    }
}

fn request_shutdown(state: &CallbackServerState) {
    if let Ok(mut guard) = state.shutdown_sender.lock() {
        if let Some(sender) = guard.take() {
            let _ = sender.send(());
        }
    }
}

fn persist_callback_code(state: &CallbackServerState, code: &str) -> Result<(), CodexError> {
    let mut pending: CodexPendingOAuthState = read_json_file_opt(&state.pending_file_path)?
        .ok_or_else(|| {
            CodexError::OAuth(format!(
                "OAuth pending state not found: {}",
                state.pending_file_path.display()
            ))
        })?;

    if pending.login_id != state.login_id {
        return Err(CodexError::AuthState(format!(
            "Stale login id: expected {} got {}",
            pending.login_id, state.login_id
        )));
    }

    pending.code = Some(code.to_string());
    write_json_atomic(&state.pending_file_path, &pending)
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
    timeout_secs: u64,
) -> Result<oneshot::Receiver<CallbackResult>, CodexError> {
    let (result_sender, result_receiver) = oneshot::channel();
    let (shutdown_sender, shutdown_receiver) = oneshot::channel();

    let state = Arc::new(CallbackServerState {
        login_id,
        expected_state,
        code_verifier,
        emitter,
        pending_file_path,
        shutdown_sender: Arc::new(StdMutex::new(Some(shutdown_sender))),
        result_sender: Arc::new(StdMutex::new(Some(result_sender))),
    });

    let timeout_state = state.clone();
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
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(timeout_secs)).await;
                send_result_once(&timeout_state, CallbackResult::Timeout);
                timeout_state.emitter.emit(OAuthEvent::LoginTimeout {
                    login_id: timeout_state.login_id.clone(),
                });
                request_shutdown(&timeout_state);
            });

            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_receiver.await;
                })
                .await
                .unwrap();
        });
    });

    Ok(result_receiver)
}
