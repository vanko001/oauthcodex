use crate::domain::codex_models::OAuthTokenResponse;
use crate::error::CodexError;
use reqwest::Client;
use serde::de::Error as _;
use std::collections::HashMap;
use std::time::Duration;

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Result<Self, CodexError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| CodexError::Http(format!("Failed to build HTTP client: {e}")))?;
        Ok(Self { client })
    }

    pub async fn exchange_code_for_tokens(
        &self,
        token_endpoint: &str,
        code: &str,
        code_verifier: &str,
        redirect_uri: &str,
        client_id: &str,
    ) -> Result<OAuthTokenResponse, CodexError> {
        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", code);
        params.insert("code_verifier", code_verifier);
        params.insert("redirect_uri", redirect_uri);
        params.insert("client_id", client_id);

        let response = self
            .client
            .post(token_endpoint)
            .form(&params)
            .send()
            .await
            .map_err(|e| CodexError::Http(format!("Token exchange request failed: {e}")))?;

        let status = response.status();
        let body_text = response
            .text()
            .await
            .map_err(|e| CodexError::Http(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            match serde_json::from_str::<OAuthTokenResponse>(&body_text) {
                Ok(resp) if resp.error.is_some() => {
                    return Err(CodexError::OAuth(format!(
                        "Token exchange error: {} - {}",
                        resp.error.unwrap_or_default(),
                        resp.error_description.unwrap_or_default()
                    )));
                }
                _ => {
                    return Err(CodexError::Http(format!(
                        "Token exchange HTTP {}: body_len={}",
                        status.as_u16(),
                        body_text.len()
                    )));
                }
            }
        }

        let token_response: OAuthTokenResponse = serde_json::from_str(&body_text).map_err(|e| {
            CodexError::Json(serde_json::Error::custom(format!(
                "Failed to parse token response: {e}"
            )))
        })?;

        Ok(token_response)
    }

    pub async fn refresh_access_token(
        &self,
        token_endpoint: &str,
        refresh_token: &str,
        client_id: &str,
    ) -> Result<OAuthTokenResponse, CodexError> {
        let mut params = HashMap::new();
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", refresh_token);
        params.insert("client_id", client_id);

        let response = self
            .client
            .post(token_endpoint)
            .form(&params)
            .send()
            .await
            .map_err(|e| CodexError::Http(format!("Token refresh request failed: {e}")))?;

        let status = response.status();
        let body_text = response
            .text()
            .await
            .map_err(|e| CodexError::Http(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            match serde_json::from_str::<OAuthTokenResponse>(&body_text) {
                Ok(resp) if resp.error.is_some() => {
                    return Err(CodexError::Token(format!(
                        "Token refresh error: {} - {}",
                        resp.error.unwrap_or_default(),
                        resp.error_description.unwrap_or_default()
                    )));
                }
                _ => {
                    return Err(CodexError::Http(format!(
                        "Token refresh HTTP {}: body_len={}",
                        status.as_u16(),
                        body_text.len()
                    )));
                }
            }
        }

        let token_response: OAuthTokenResponse = serde_json::from_str(&body_text).map_err(|e| {
            CodexError::Json(serde_json::Error::custom(format!(
                "Failed to parse refresh response: {e}"
            )))
        })?;

        Ok(token_response)
    }

    pub async fn get_usage(&self, url: &str, bearer_token: &str) -> Result<String, CodexError> {
        self.get_usage_for_account(url, bearer_token, None).await
    }

    pub async fn get_usage_for_account(
        &self,
        url: &str,
        bearer_token: &str,
        account_id: Option<&str>,
    ) -> Result<String, CodexError> {
        let mut request = self
            .client
            .get(url)
            .bearer_auth(bearer_token)
            .header(reqwest::header::ACCEPT, "application/json");

        if let Some(account_id) = account_id.filter(|id| !id.is_empty()) {
            request = request.header("ChatGPT-Account-Id", account_id);
        }

        let response = request
            .send()
            .await
            .map_err(|e| CodexError::Http(format!("Usage API request failed: {e}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| CodexError::Http(format!("Failed to read usage response: {e}")))?;

        if !status.is_success() {
            return Err(CodexError::Http(format!(
                "Usage API HTTP {}: body_len={}",
                status.as_u16(),
                body.len()
            )));
        }

        Ok(body)
    }

    pub async fn get_account_profile(
        &self,
        url: &str,
        bearer_token: &str,
    ) -> Result<String, CodexError> {
        let response = self
            .client
            .get(url)
            .bearer_auth(bearer_token)
            .send()
            .await
            .map_err(|e| CodexError::Http(format!("Profile API request failed: {e}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| CodexError::Http(format!("Failed to read profile response: {e}")))?;

        if !status.is_success() {
            return Err(CodexError::Http(format!(
                "Profile API HTTP {}: body_len={}",
                status.as_u16(),
                body.len()
            )));
        }

        Ok(body)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("default HTTP client")
    }
}
