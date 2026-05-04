use crate::error::CodexError;
use url::Url;

const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

pub struct ApiKeyValidation {
    pub is_valid: bool,
    pub error: Option<String>,
    pub normalized_base_url: String,
    pub provider_id: String,
    pub provider_mode: String,
}

pub fn validate_api_key(
    api_key: &str,
    base_url: &str,
    provider_name: Option<&str>,
) -> Result<ApiKeyValidation, CodexError> {
    if api_key.trim().is_empty() {
        return Ok(ApiKeyValidation {
            is_valid: false,
            error: Some("API key cannot be empty".into()),
            normalized_base_url: String::new(),
            provider_id: String::new(),
            provider_mode: "openai".into(),
        });
    }

    if api_key.trim().starts_with("http://") || api_key.trim().starts_with("https://") {
        return Ok(ApiKeyValidation {
            is_valid: false,
            error: Some("URL pasted into API key field. Please paste only the API key.".into()),
            normalized_base_url: String::new(),
            provider_id: String::new(),
            provider_mode: "openai".into(),
        });
    }

    let normalized_url = normalize_base_url(base_url)?;

    let is_openai_default = normalized_url == DEFAULT_OPENAI_BASE_URL;
    let provider_id = derive_provider_id(&normalized_url, provider_name, is_openai_default);
    let provider_mode = if is_openai_default {
        "openai"
    } else {
        "custom"
    };

    Ok(ApiKeyValidation {
        is_valid: true,
        error: None,
        normalized_base_url: normalized_url,
        provider_id,
        provider_mode: provider_mode.into(),
    })
}

pub fn normalize_base_url(base_url: &str) -> Result<String, CodexError> {
    let trimmed = base_url.trim();

    if trimmed.is_empty() {
        return Ok(DEFAULT_OPENAI_BASE_URL.to_string());
    }

    let with_scheme = if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        format!("https://{}", trimmed)
    } else {
        trimmed.to_string()
    };

    let parsed = Url::parse(&with_scheme)
        .map_err(|_| CodexError::ApiKey(format!("Invalid base URL: {}", trimmed)))?;

    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(CodexError::ApiKey(format!(
            "Invalid base URL scheme: {}",
            scheme
        )));
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| CodexError::ApiKey("Base URL missing host".into()))?;

    let mut normalized = format!("{}://{}", scheme, host);

    if let Some(port) = parsed.port() {
        normalized = format!("{}:{port}", normalized);
    }

    let path = parsed.path().trim_end_matches('/');
    if !path.is_empty() && path != "/" {
        normalized = format!("{normalized}{path}");
    }

    let default_path = "/v1";
    if !normalized.ends_with(default_path) {
        normalized = format!("{normalized}{default_path}");
    }

    Ok(normalized)
}

pub fn derive_provider_id(
    base_url: &str,
    provider_name: Option<&str>,
    is_openai_default: bool,
) -> String {
    if is_openai_default {
        "cmp_openai_default".into()
    } else if let Some(name) = provider_name {
        let sanitized: String = name
            .chars()
            .map(|c| if c == ' ' { '_' } else { c })
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .take(40)
            .collect::<String>()
            .to_lowercase();

        if sanitized.is_empty() {
            let host_id = extract_host_id(base_url);
            format!("cmp_{host_id}")
        } else {
            format!("cmp_{sanitized}")
        }
    } else {
        let host_id = extract_host_id(base_url);
        format!("cmp_{host_id}")
    }
}

fn extract_host_id(base_url: &str) -> String {
    Url::parse(base_url)
        .ok()
        .and_then(|u| {
            u.host_str().map(|h| {
                h.replace(['.', ':'], "_")
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '_')
                    .take(40)
                    .collect::<String>()
                    .to_lowercase()
            })
        })
        .unwrap_or_else(|| "custom".into())
}

pub fn generate_api_key_id() -> String {
    let uuid_str = uuid::Uuid::new_v4().to_string().replace('-', "_");
    format!("cmk_{}", &uuid_str[..16])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_api_key() {
        let result = validate_api_key("", "https://api.openai.com/v1", None).unwrap();
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("empty"));
    }

    #[test]
    fn test_url_in_api_key_field() {
        let result =
            validate_api_key("https://evil.com/key", "https://api.openai.com/v1", None).unwrap();
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("URL pasted"));
    }

    #[test]
    fn test_valid_openai_default() {
        let result = validate_api_key("sk-test123", "https://api.openai.com/v1", None).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.normalized_base_url, "https://api.openai.com/v1");
        assert_eq!(result.provider_id, "cmp_openai_default");
        assert_eq!(result.provider_mode, "openai");
    }

    #[test]
    fn test_empty_base_url_defaults_to_openai() {
        let result = validate_api_key("sk-test123", "", None).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.normalized_base_url, "https://api.openai.com/v1");
    }

    #[test]
    fn test_custom_provider_derives_id() {
        let result = validate_api_key(
            "sk-custom",
            "https://custom-llm.example.com",
            Some("My LLM"),
        )
        .unwrap();
        assert!(result.is_valid);
        assert_eq!(
            result.normalized_base_url,
            "https://custom-llm.example.com/v1"
        );
        assert_eq!(result.provider_id, "cmp_my_llm");
        assert_eq!(result.provider_mode, "custom");
    }

    #[test]
    fn test_base_url_normalization_adds_v1() {
        let url = normalize_base_url("https://api.groq.com").unwrap();
        assert_eq!(url, "https://api.groq.com/v1");
    }

    #[test]
    fn test_base_url_normalization_no_scheme() {
        let url = normalize_base_url("api.anthropic.com").unwrap();
        assert_eq!(url, "https://api.anthropic.com/v1");
    }

    #[test]
    fn test_base_url_with_path_preserved() {
        let url = normalize_base_url("https://ai.example.com/openai").unwrap();
        assert_eq!(url, "https://ai.example.com/openai/v1");
    }

    #[test]
    fn test_invalid_base_url_rejected() {
        let result = normalize_base_url("not a valid url at all !!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_same_key_provider_combination() {
        let r1 = validate_api_key("sk-same", "https://api.openai.com/v1", None).unwrap();
        let r2 = validate_api_key("sk-same", "https://api.openai.com/v1", None).unwrap();
        assert_eq!(r1.provider_id, r2.provider_id);
        assert_eq!(r1.normalized_base_url, r2.normalized_base_url);
    }
}
