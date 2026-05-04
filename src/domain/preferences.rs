use crate::adapters::fs_store::CodexPaths;

pub struct PreferenceStore {
    pub paths: CodexPaths,
}

impl PreferenceStore {
    pub fn new(paths: CodexPaths) -> Self {
        Self { paths }
    }
}

pub fn overview_layout_key() -> &'static str {
    "agtools.codex.accounts.overview_layout_mode"
}

pub fn custom_sort_key() -> &'static str {
    "agtools.codex.accounts.custom_sort_order.v1"
}

pub fn local_access_expanded_key() -> &'static str {
    "agtools.codex.local_access_entry_expanded.v1"
}

pub fn code_review_quota_key() -> &'static str {
    "agtools.codex_show_code_review_quota"
}

pub fn api_switch_dismissed_key() -> &'static str {
    "codexApiSwitchVisibilityNoticeDismissed"
}

pub fn current_refresh_map_key() -> &'static str {
    "agtools.current_account_refresh_minutes.v1"
}

pub fn accounts_cache_key() -> &'static str {
    "agtools.codex.accounts.cache"
}

pub fn current_account_key() -> &'static str {
    "agtools.codex.accounts.current"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overview_layout_key() {
        assert_eq!(
            overview_layout_key(),
            "agtools.codex.accounts.overview_layout_mode"
        );
    }

    #[test]
    fn test_custom_sort_key() {
        assert_eq!(
            custom_sort_key(),
            "agtools.codex.accounts.custom_sort_order.v1"
        );
    }

    #[test]
    fn test_local_access_expanded_key() {
        assert_eq!(
            local_access_expanded_key(),
            "agtools.codex.local_access_entry_expanded.v1"
        );
    }

    #[test]
    fn test_code_review_quota_key() {
        assert_eq!(
            code_review_quota_key(),
            "agtools.codex_show_code_review_quota"
        );
    }

    #[test]
    fn test_api_switch_dismissed_key() {
        assert_eq!(
            api_switch_dismissed_key(),
            "codexApiSwitchVisibilityNoticeDismissed"
        );
    }

    #[test]
    fn test_current_refresh_map_key() {
        assert_eq!(
            current_refresh_map_key(),
            "agtools.current_account_refresh_minutes.v1"
        );
    }

    #[test]
    fn test_accounts_cache_key() {
        assert_eq!(accounts_cache_key(), "agtools.codex.accounts.cache");
    }

    #[test]
    fn test_current_account_key() {
        assert_eq!(current_account_key(), "agtools.codex.accounts.current");
    }
}
