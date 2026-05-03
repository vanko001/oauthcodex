use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodexError {
    #[error("OAuth error: {0}")]
    OAuth(String),

    #[error("Token error: {0}")]
    Token(String),

    #[error("Auth state error: {0}")]
    AuthState(String),

    #[error("Account store error: {0}")]
    AccountStore(String),

    #[error("Import error: {0}")]
    Import(String),

    #[error("Export error: {0}")]
    Export(String),

    #[error("API key error: {0}")]
    ApiKey(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Group error: {0}")]
    Group(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Data transfer error: {0}")]
    DataTransfer(String),

    #[error("Preference error: {0}")]
    Preference(String),

    #[error("Quota error: {0}")]
    Quota(String),

    #[error("Local access error: {0}")]
    LocalAccess(String),

    #[error("Instance error: {0}")]
    Instance(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Wakeup error: {0}")]
    Wakeup(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("JWT error: {0}")]
    Jwt(String),

    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Cancelled: {0}")]
    Cancelled(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),
}

impl CodexError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            CodexError::Http(_)
                | CodexError::RateLimited(_)
                | CodexError::Timeout(_)
                | CodexError::Io(_)
        )
    }

    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            CodexError::Unauthorized(_) | CodexError::Forbidden(_) | CodexError::Token(_)
        )
    }

    pub fn requires_reauth(&self) -> bool {
        matches!(
            self,
            CodexError::Token(_) | CodexError::Unauthorized(_) | CodexError::InvalidState(_)
        )
    }
}
