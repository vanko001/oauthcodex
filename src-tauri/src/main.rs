use oauthcodex::adapters::app_bridge::CodexAppBridge;
use oauthcodex::adapters::fs_store::CodexPaths;
use oauthcodex::domain::account::ImportResult;
use oauthcodex::domain::codex_models::{
    CodexAccount, CodexAccountGroup, CodexAccountGroupList, CodexApiProviderMode,
    CodexModelProvider, CodexModelProviderList, CodexQuota, LocalAccessStateSnapshot,
    RoutingStrategy,
};
use oauthcodex::error::CodexError;
use serde_json::{json, Value};

fn bridge() -> Result<CodexAppBridge, String> {
    let paths = CodexPaths::new().map_err(error_to_string)?;
    paths.ensure_dirs().map_err(error_to_string)?;
    Ok(CodexAppBridge::new(paths))
}

fn error_to_string(error: CodexError) -> String {
    error.to_string()
}

fn result<T>(value: Result<T, CodexError>) -> Result<T, String> {
    value.map_err(error_to_string)
}

fn now_epoch() -> i64 {
    chrono_like_now()
}

fn chrono_like_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

struct MinimalInstance {
    id: String,
    name: String,
    user_data_dir: Option<String>,
    working_dir: Option<String>,
    extra_args: Option<String>,
    bind_account_id: Option<String>,
    launch_mode: Option<String>,
    running: bool,
}

fn minimal_instance(input: MinimalInstance) -> Value {
    let now = now_epoch();
    json!({
        "id": input.id,
        "name": input.name,
        "userDataDir": input.user_data_dir.unwrap_or_default(),
        "workingDir": input.working_dir,
        "extraArgs": input.extra_args.unwrap_or_default(),
        "bindAccountId": input.bind_account_id,
        "launchMode": input.launch_mode.unwrap_or_else(|| "auto".to_string()),
        "createdAt": now,
        "lastLaunchedAt": if input.running { Some(now) } else { None },
        "lastPid": Value::Null,
        "running": input.running,
        "initialized": true,
        "isDefault": false,
        "followLocalAccount": true
    })
}

fn cli_status() -> Value {
    json!({
        "available": false,
        "message": "Codex CLI runtime is not configured in this app build.",
        "required_runtime_paths": [],
        "checked_at": now_epoch(),
        "install_hints": []
    })
}

fn wakeup_state(enabled: bool, tasks: Vec<Value>, model_presets: Vec<Value>) -> Value {
    json!({
        "enabled": enabled,
        "tasks": tasks,
        "model_presets": model_presets,
        "model_preset_migrations": []
    })
}

#[cfg(target_os = "linux")]
fn configure_linux_webkit() {
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
}

#[cfg(not(target_os = "linux"))]
fn configure_linux_webkit() {}

fn parse_groups(data: String) -> Result<Vec<CodexAccountGroup>, String> {
    let value: Value = serde_json::from_str(&data).map_err(|error| error.to_string())?;
    if value.is_array() {
        serde_json::from_value(value).map_err(|error| error.to_string())
    } else {
        serde_json::from_value::<CodexAccountGroupList>(value)
            .map(|list| list.groups)
            .map_err(|error| error.to_string())
    }
}

fn parse_providers(data: String) -> Result<Vec<CodexModelProvider>, String> {
    let value: Value = serde_json::from_str(&data).map_err(|error| error.to_string())?;
    if value.is_array() {
        serde_json::from_value(value).map_err(|error| error.to_string())
    } else {
        serde_json::from_value::<CodexModelProviderList>(value)
            .map(|list| list.providers)
            .map_err(|error| error.to_string())
    }
}

#[tauri::command]
fn list_codex_accounts() -> Result<Vec<CodexAccount>, String> {
    result(bridge()?.list_codex_accounts())
}

#[tauri::command]
fn get_current_codex_account() -> Result<Option<CodexAccount>, String> {
    result(bridge()?.get_current_codex_account())
}

#[tauri::command]
fn switch_codex_account(account_id: String) -> Result<CodexAccount, String> {
    result(bridge()?.switch_codex_account(account_id))
}

#[tauri::command]
fn delete_codex_account(account_id: String) -> Result<(), String> {
    result(bridge()?.delete_codex_account(account_id))
}

#[tauri::command]
fn delete_codex_accounts(account_ids: Vec<String>) -> Result<(), String> {
    result(bridge()?.delete_codex_accounts(account_ids))
}

#[tauri::command]
fn import_codex_from_local() -> Result<Option<CodexAccount>, String> {
    result(bridge()?.import_codex_from_local())
}

#[tauri::command]
fn import_codex_from_json(json_content: String) -> Result<Vec<CodexAccount>, String> {
    result(bridge()?.import_codex_from_json(json_content))
}

#[tauri::command]
fn import_codex_from_files(file_paths: Vec<String>) -> Result<ImportResult, String> {
    result(bridge()?.import_codex_from_files(file_paths))
}

#[tauri::command]
fn export_codex_accounts(account_ids: Vec<String>) -> Result<String, String> {
    result(bridge()?.export_codex_accounts(account_ids))
}

#[tauri::command]
fn add_codex_account_with_token(
    id_token: String,
    access_token: String,
    refresh_token: Option<String>,
) -> Result<CodexAccount, String> {
    result(bridge()?.add_codex_account_with_token(id_token, access_token, refresh_token))
}

#[tauri::command]
fn add_codex_account_with_api_key(
    api_key: String,
    api_base_url: Option<String>,
    api_provider_mode: Option<CodexApiProviderMode>,
    api_provider_id: Option<String>,
    api_provider_name: Option<String>,
) -> Result<CodexAccount, String> {
    result(bridge()?.add_codex_account_with_api_key(
        api_key,
        api_base_url,
        api_provider_mode,
        api_provider_id,
        api_provider_name,
    ))
}

#[tauri::command]
fn update_codex_api_key_credentials(
    account_id: String,
    api_key: String,
    api_base_url: Option<String>,
    api_provider_mode: Option<CodexApiProviderMode>,
    api_provider_id: Option<String>,
    api_provider_name: Option<String>,
) -> Result<CodexAccount, String> {
    result(bridge()?.update_codex_api_key_credentials(
        account_id,
        api_key,
        api_base_url,
        api_provider_mode,
        api_provider_id,
        api_provider_name,
    ))
}

#[tauri::command]
fn update_codex_account_name(account_id: String, name: String) -> Result<CodexAccount, String> {
    result(bridge()?.update_codex_account_name(account_id, name))
}

#[tauri::command]
fn update_codex_account_tags(
    account_id: String,
    tags: Vec<String>,
) -> Result<CodexAccount, String> {
    result(bridge()?.update_codex_account_tags(account_id, tags))
}

#[tauri::command]
fn refresh_codex_quota(account_id: String) -> Result<CodexQuota, String> {
    let account = result(bridge()?.switch_codex_account(account_id))?;
    Ok(account.quota.map(|quota| *quota).unwrap_or_default())
}

#[tauri::command]
fn refresh_all_codex_quotas() -> u32 {
    0
}

#[tauri::command]
fn refresh_codex_account_profile(account_id: String) -> Result<CodexAccount, String> {
    let account = result(bridge()?.switch_codex_account(account_id))?;
    Ok(account)
}

#[tauri::command]
fn codex_oauth_login_start() -> Result<Value, String> {
    result(bridge()?.codex_oauth_login_start())
}

#[tauri::command]
async fn codex_oauth_login_completed(login_id: String) -> Result<CodexAccount, String> {
    result(bridge()?.codex_oauth_login_completed(login_id).await)
}

#[tauri::command]
fn codex_oauth_login_cancel(login_id: Option<String>) -> Result<(), String> {
    result(bridge()?.codex_oauth_login_cancel(login_id))
}

#[tauri::command]
fn codex_oauth_submit_callback_url(login_id: String, callback_url: String) -> Result<(), String> {
    result(bridge()?.codex_oauth_submit_callback_url(login_id, callback_url))
}

#[tauri::command]
fn is_codex_oauth_port_in_use() -> Result<bool, String> {
    let paths = CodexPaths::new().map_err(error_to_string)?;
    let service = oauthcodex::domain::oauth::OAuthService::new(paths);
    service
        .check_port_available(1455)
        .map(|available| !available)
        .map_err(error_to_string)
}

#[tauri::command]
fn close_codex_oauth_port() -> u32 {
    0
}

#[tauri::command]
fn get_codex_config_toml_path() -> Result<String, String> {
    result(bridge()?.get_codex_config_toml_path())
}

#[tauri::command]
fn open_codex_config_toml() -> Result<(), String> {
    let path = bridge()?
        .get_codex_config_toml_path()
        .map_err(error_to_string)?;
    std::process::Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_codex_quick_config() -> Result<Value, String> {
    result(
        bridge()?
            .get_codex_quick_config()
            .and_then(|config| serde_json::to_value(config).map_err(CodexError::Json)),
    )
}

#[tauri::command]
fn save_codex_quick_config(
    model_context_window: Option<u64>,
    auto_compact_token_limit: Option<u64>,
) -> Result<Value, String> {
    result(
        bridge()?
            .save_codex_quick_config(model_context_window, auto_compact_token_limit)
            .and_then(|config| serde_json::to_value(config).map_err(CodexError::Json)),
    )
}

#[tauri::command]
fn get_general_config() -> Result<Value, String> {
    result(bridge()?.get_general_config())
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
fn save_general_config(
    codex_auto_refresh_minutes: Option<i32>,
    codex_startup_wakeup_enabled: Option<bool>,
    codex_startup_wakeup_delay_seconds: Option<u64>,
    codex_app_path: Option<String>,
    codex_specified_app_path: Option<String>,
    codex_launch_on_switch: Option<bool>,
    codex_restart_specified_app_on_switch: Option<bool>,
    codex_local_access_entry_visible: Option<bool>,
    codex_auto_switch_enabled: Option<bool>,
    codex_auto_switch_primary_threshold: Option<i32>,
    codex_auto_switch_secondary_threshold: Option<i32>,
    codex_auto_switch_account_scope_mode: Option<String>,
    codex_auto_switch_selected_account_ids: Option<Vec<String>>,
    codex_quota_alert_enabled: Option<bool>,
    codex_quota_alert_threshold: Option<i32>,
    codex_quota_alert_primary_threshold: Option<i32>,
    codex_quota_alert_secondary_threshold: Option<i32>,
    codex_quota_alert_cooldown_minutes: Option<u64>,
) -> Result<(), String> {
    let config = json!({
        "codex_auto_refresh_minutes": codex_auto_refresh_minutes,
        "codex_startup_wakeup_enabled": codex_startup_wakeup_enabled,
        "codex_startup_wakeup_delay_seconds": codex_startup_wakeup_delay_seconds,
        "codex_app_path": codex_app_path,
        "codex_specified_app_path": codex_specified_app_path,
        "codex_launch_on_switch": codex_launch_on_switch,
        "codex_restart_specified_app_on_switch": codex_restart_specified_app_on_switch,
        "codex_local_access_entry_visible": codex_local_access_entry_visible,
        "codex_auto_switch_enabled": codex_auto_switch_enabled,
        "codex_auto_switch_primary_threshold": codex_auto_switch_primary_threshold,
        "codex_auto_switch_secondary_threshold": codex_auto_switch_secondary_threshold,
        "codex_auto_switch_account_scope_mode": codex_auto_switch_account_scope_mode,
        "codex_auto_switch_selected_account_ids": codex_auto_switch_selected_account_ids,
        "codex_quota_alert_enabled": codex_quota_alert_enabled,
        "codex_quota_alert_threshold": codex_quota_alert_threshold,
        "codex_quota_alert_primary_threshold": codex_quota_alert_primary_threshold,
        "codex_quota_alert_secondary_threshold": codex_quota_alert_secondary_threshold,
        "codex_quota_alert_cooldown_minutes": codex_quota_alert_cooldown_minutes
    });
    result(bridge()?.save_general_config(config))
}

#[tauri::command]
fn codex_local_access_get_state() -> Result<LocalAccessStateSnapshot, String> {
    result(bridge()?.codex_local_access_get_state())
}

#[tauri::command]
fn codex_local_access_save_accounts(
    account_ids: Vec<String>,
    restrict_free_accounts: bool,
) -> Result<LocalAccessStateSnapshot, String> {
    result(bridge()?.codex_local_access_save_accounts(account_ids, restrict_free_accounts))
}

#[tauri::command]
fn codex_local_access_remove_account(
    account_id: String,
) -> Result<LocalAccessStateSnapshot, String> {
    result(bridge()?.codex_local_access_remove_account(account_id))
}

#[tauri::command]
fn codex_local_access_rotate_api_key() -> Result<LocalAccessStateSnapshot, String> {
    result(bridge()?.codex_local_access_rotate_api_key())
}

#[tauri::command]
fn codex_local_access_clear_stats() -> Result<LocalAccessStateSnapshot, String> {
    result(bridge()?.codex_local_access_clear_stats())
}

#[tauri::command]
fn codex_local_access_prepare_restart() -> Result<LocalAccessStateSnapshot, String> {
    result(bridge()?.codex_local_access_prepare_restart())
}

#[tauri::command]
fn codex_local_access_kill_port() -> Result<Value, String> {
    let state = result(bridge()?.codex_local_access_get_state())?;
    Ok(json!({ "killedCount": 0, "state": state }))
}

#[tauri::command]
fn codex_local_access_update_port(port: u16) -> Result<LocalAccessStateSnapshot, String> {
    result(bridge()?.codex_local_access_update_port(port))
}

#[tauri::command]
fn codex_local_access_update_routing_strategy(
    strategy: RoutingStrategy,
) -> Result<LocalAccessStateSnapshot, String> {
    result(bridge()?.codex_local_access_update_routing_strategy(strategy))
}

#[tauri::command]
fn codex_local_access_set_enabled(enabled: bool) -> Result<LocalAccessStateSnapshot, String> {
    result(bridge()?.codex_local_access_set_enabled(enabled))
}

#[tauri::command]
fn codex_local_access_activate() -> Result<LocalAccessStateSnapshot, String> {
    result(bridge()?.codex_local_access_activate())
}

#[tauri::command]
fn load_codex_account_groups() -> Result<Vec<CodexAccountGroup>, String> {
    result(bridge()?.load_codex_account_groups())
}

#[tauri::command]
fn save_codex_account_groups(data: String) -> Result<(), String> {
    let groups = parse_groups(data)?;
    result(bridge()?.save_codex_account_groups(groups))
}

#[tauri::command]
fn load_codex_model_providers() -> Result<Vec<CodexModelProvider>, String> {
    result(bridge()?.load_codex_model_providers())
}

#[tauri::command]
fn save_codex_model_providers(data: String) -> Result<(), String> {
    let providers = parse_providers(data)?;
    result(bridge()?.save_codex_model_providers(providers))
}

#[tauri::command]
fn codex_get_instance_defaults() -> Result<Value, String> {
    let paths = CodexPaths::new().map_err(error_to_string)?;
    let root = paths
        .home
        .join(".antigravity_cockpit")
        .join("codex_instances");
    Ok(json!({
        "rootDir": root,
        "defaultUserDataDir": paths.codex_dir
    }))
}

#[tauri::command]
fn codex_list_instances() -> Vec<Value> {
    Vec::new()
}

#[tauri::command]
fn codex_create_instance(
    name: String,
    user_data_dir: String,
    working_dir: Option<String>,
    extra_args: Option<String>,
    bind_account_id: Option<String>,
    launch_mode: Option<String>,
) -> Value {
    minimal_instance(MinimalInstance {
        id: format!("inst_codex_{}", now_epoch()),
        name,
        user_data_dir: Some(user_data_dir),
        working_dir,
        extra_args,
        bind_account_id,
        launch_mode,
        running: false,
    })
}

#[tauri::command]
fn codex_update_instance(
    instance_id: String,
    name: Option<String>,
    working_dir: Option<String>,
    extra_args: Option<String>,
    bind_account_id: Option<String>,
    launch_mode: Option<String>,
) -> Value {
    minimal_instance(MinimalInstance {
        id: instance_id,
        name: name.unwrap_or_else(|| "Codex Instance".to_string()),
        user_data_dir: None,
        working_dir,
        extra_args,
        bind_account_id,
        launch_mode,
        running: false,
    })
}

#[tauri::command]
fn codex_delete_instance(_instance_id: String) {}

#[tauri::command]
fn codex_start_instance(instance_id: String) -> Value {
    minimal_instance(MinimalInstance {
        id: instance_id,
        name: "Codex Instance".to_string(),
        user_data_dir: None,
        working_dir: None,
        extra_args: None,
        bind_account_id: None,
        launch_mode: Some("auto".to_string()),
        running: true,
    })
}

#[tauri::command]
fn codex_stop_instance(instance_id: String) -> Value {
    minimal_instance(MinimalInstance {
        id: instance_id,
        name: "Codex Instance".to_string(),
        user_data_dir: None,
        working_dir: None,
        extra_args: None,
        bind_account_id: None,
        launch_mode: Some("auto".to_string()),
        running: false,
    })
}

#[tauri::command]
fn codex_close_all_instances() {}

#[tauri::command]
fn codex_open_instance_window(_instance_id: String) {}

#[tauri::command]
fn codex_get_instance_quick_config(_instance_id: String) -> Result<Value, String> {
    get_codex_quick_config()
}

#[tauri::command]
fn codex_save_instance_quick_config(
    _instance_id: String,
    model_context_window: Option<u64>,
    auto_compact_token_limit: Option<u64>,
) -> Result<Value, String> {
    save_codex_quick_config(model_context_window, auto_compact_token_limit)
}

#[tauri::command]
fn codex_open_instance_config_toml(_instance_id: String) -> Result<(), String> {
    open_codex_config_toml()
}

#[tauri::command]
fn codex_get_instance_launch_command(instance_id: String) -> Value {
    json!({
        "instanceId": instance_id,
        "userDataDir": "",
        "launchCommand": "codex"
    })
}

#[tauri::command]
fn codex_execute_instance_launch_command(
    _instance_id: String,
    _terminal: Option<String>,
) -> String {
    "Process adapter is not implemented in this app build.".to_string()
}

#[tauri::command]
fn codex_sync_threads_across_instances() -> Value {
    json!({ "message": "No instances configured." })
}

#[tauri::command]
fn codex_repair_session_visibility_across_instances() -> Value {
    json!({ "message": "No sessions found." })
}

#[tauri::command]
fn codex_list_sessions_across_instances() -> Vec<Value> {
    Vec::new()
}

#[tauri::command]
fn codex_get_session_token_stats_across_instances(_session_ids: Vec<String>) -> Vec<Value> {
    Vec::new()
}

#[tauri::command]
fn codex_move_sessions_to_trash_across_instances(_session_ids: Vec<String>) -> Value {
    json!({ "message": "No sessions moved." })
}

#[tauri::command]
fn codex_list_trashed_sessions_across_instances() -> Vec<Value> {
    Vec::new()
}

#[tauri::command]
fn codex_restore_sessions_from_trash_across_instances(_session_ids: Vec<String>) -> Value {
    json!({ "message": "No sessions restored." })
}

#[tauri::command]
fn codex_wakeup_get_cli_status() -> Value {
    cli_status()
}

#[tauri::command]
fn codex_wakeup_update_runtime_config(
    _codex_cli_path: Option<String>,
    _node_path: Option<String>,
) -> Value {
    cli_status()
}

#[tauri::command]
fn codex_wakeup_get_overview() -> Value {
    json!({
        "runtime": cli_status(),
        "state": wakeup_state(false, Vec::new(), Vec::new()),
        "history": []
    })
}

#[tauri::command]
fn codex_wakeup_save_state(
    enabled: bool,
    tasks: Vec<Value>,
    model_presets: Vec<Value>,
    _model_preset_migrations: Option<Vec<String>>,
) -> Value {
    wakeup_state(enabled, tasks, model_presets)
}

#[tauri::command]
fn codex_wakeup_load_history() -> Vec<Value> {
    Vec::new()
}

#[tauri::command]
fn codex_wakeup_clear_history() {}

#[tauri::command]
fn codex_wakeup_test(
    _account_ids: Vec<String>,
    run_id: String,
    _prompt: Option<String>,
    _model: Option<String>,
    _model_display_name: Option<String>,
    _model_reasoning_effort: Option<String>,
    _cancel_scope_id: Option<String>,
) -> Value {
    json!({
        "run_id": run_id,
        "runtime": cli_status(),
        "records": [],
        "success_count": 0,
        "failure_count": 0
    })
}

#[tauri::command]
fn codex_wakeup_run_task(task_id: String, run_id: String) -> Value {
    json!({
        "run_id": run_id,
        "runtime": cli_status(),
        "records": [],
        "success_count": 0,
        "failure_count": 1,
        "task_id": task_id
    })
}

#[tauri::command]
fn codex_wakeup_cancel_scope(_cancel_scope_id: String) {}

#[tauri::command]
fn codex_wakeup_release_scope(_cancel_scope_id: String) {}

fn main() {
    configure_linux_webkit();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_codex_accounts,
            get_current_codex_account,
            switch_codex_account,
            delete_codex_account,
            delete_codex_accounts,
            import_codex_from_local,
            import_codex_from_json,
            import_codex_from_files,
            export_codex_accounts,
            add_codex_account_with_token,
            add_codex_account_with_api_key,
            update_codex_api_key_credentials,
            update_codex_account_name,
            update_codex_account_tags,
            refresh_codex_quota,
            refresh_all_codex_quotas,
            refresh_codex_account_profile,
            codex_oauth_login_start,
            codex_oauth_login_completed,
            codex_oauth_login_cancel,
            codex_oauth_submit_callback_url,
            is_codex_oauth_port_in_use,
            close_codex_oauth_port,
            get_codex_config_toml_path,
            open_codex_config_toml,
            get_codex_quick_config,
            save_codex_quick_config,
            get_general_config,
            save_general_config,
            codex_local_access_get_state,
            codex_local_access_save_accounts,
            codex_local_access_remove_account,
            codex_local_access_rotate_api_key,
            codex_local_access_clear_stats,
            codex_local_access_prepare_restart,
            codex_local_access_kill_port,
            codex_local_access_update_port,
            codex_local_access_update_routing_strategy,
            codex_local_access_set_enabled,
            codex_local_access_activate,
            load_codex_account_groups,
            save_codex_account_groups,
            load_codex_model_providers,
            save_codex_model_providers,
            codex_get_instance_defaults,
            codex_list_instances,
            codex_create_instance,
            codex_update_instance,
            codex_delete_instance,
            codex_start_instance,
            codex_stop_instance,
            codex_close_all_instances,
            codex_open_instance_window,
            codex_get_instance_quick_config,
            codex_save_instance_quick_config,
            codex_open_instance_config_toml,
            codex_get_instance_launch_command,
            codex_execute_instance_launch_command,
            codex_sync_threads_across_instances,
            codex_repair_session_visibility_across_instances,
            codex_list_sessions_across_instances,
            codex_get_session_token_stats_across_instances,
            codex_move_sessions_to_trash_across_instances,
            codex_list_trashed_sessions_across_instances,
            codex_restore_sessions_from_trash_across_instances,
            codex_wakeup_get_cli_status,
            codex_wakeup_update_runtime_config,
            codex_wakeup_get_overview,
            codex_wakeup_save_state,
            codex_wakeup_load_history,
            codex_wakeup_clear_history,
            codex_wakeup_test,
            codex_wakeup_run_task,
            codex_wakeup_cancel_scope,
            codex_wakeup_release_scope
        ])
        .run(tauri::generate_context!())
        .expect("failed to run OAuth Codex app");
}
