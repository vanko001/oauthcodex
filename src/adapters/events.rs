use crate::domain::oauth::OAuthEvent;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct EventEmitter {
    #[allow(clippy::type_complexity)]
    listeners: Arc<Mutex<HashMap<String, Vec<Box<dyn Fn(OAuthEvent) + Send + 'static>>>>>,
}

impl std::fmt::Debug for EventEmitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventEmitter").finish()
    }
}

impl Clone for EventEmitter {
    fn clone(&self) -> Self {
        Self {
            listeners: Arc::clone(&self.listeners),
        }
    }
}

impl EventEmitter {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn on<F>(&self, event_name: &str, handler: F)
    where
        F: Fn(OAuthEvent) + Send + 'static,
    {
        let mut guard = self.listeners.lock().unwrap();
        guard
            .entry(event_name.to_string())
            .or_default()
            .push(Box::new(handler));
    }

    pub fn emit(&self, event: OAuthEvent) {
        let name = event.event_name().to_string();
        if let Ok(guard) = self.listeners.lock() {
            if let Some(handlers) = guard.get(&name) {
                for handler in handlers {
                    handler(event.clone());
                }
            }
        }
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::oauth::OAuthEvent;

    #[test]
    fn test_event_emitter() {
        let emitter = EventEmitter::new();
        let called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let called_clone = called.clone();

        emitter.on("codex-oauth-login-completed", move |event| {
            if let OAuthEvent::LoginCompleted { login_id, .. } = event {
                assert_eq!(login_id, "test_login");
                let mut c = called_clone.lock().unwrap();
                *c = true;
            }
        });

        emitter.emit(OAuthEvent::LoginCompleted {
            login_id: "test_login".to_string(),
            account: Box::new(crate::domain::codex_models::CodexAccount {
                id: "test".into(),
                provider: "codex".into(),
                auth_mode: crate::domain::codex_models::CodexAuthMode::OAuth,
                email: None,
                plan_type: None,
                account_id: None,
                organization_id: None,
                organizations: vec![],
                display_name: "test".into(),
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

        assert!(*called.lock().unwrap());
    }
}
