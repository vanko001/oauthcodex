use oauthcodex::adapters::fs_store::CodexPaths;
use oauthcodex::domain::account::AccountStore;
use oauthcodex::domain::local_access::LocalAccessService;
use oauthcodex::domain::oauth::OAuthService;
use oauthcodex::domain::wakeup::WakeupScheduler;
use oauthcodex::error::CodexError;
use serde_json::json;

fn runtime_paths() -> Result<CodexPaths, CodexError> {
    let paths = CodexPaths::new()?;
    paths.ensure_dirs()?;
    Ok(paths)
}

fn cmd_oauth_start() -> Result<String, CodexError> {
    let paths = runtime_paths()?;
    let svc = OAuthService::new(paths);
    let (url, _, pending) = svc.start_oauth_login(CALLBACK_PORT)?;
    Ok(json!({"auth_url": url, "login_id": pending.login_id, "state": pending.state}).to_string())
}

fn cmd_oauth_complete(args: &[String]) -> Result<String, CodexError> {
    if args.len() < 2 {
        return Err(CodexError::InvalidState(
            "Usage: oauth complete <login_id> <callback_url>".into(),
        ));
    }
    let login_id = &args[0];
    let callback_url = &args[1];

    let paths = runtime_paths()?;
    let svc = OAuthService::new(paths);
    let (code, state) = svc.parse_manual_callback(callback_url)?;
    let runtime = tokio::runtime::Runtime::new().map_err(CodexError::Io)?;
    let account = runtime.block_on(svc.complete_oauth_login_with_exchange(
        login_id,
        &[("code".to_string(), code), ("state".to_string(), state)],
    ))?;
    Ok(json!({"status": "completed", "login_id": login_id, "account_id": account.id}).to_string())
}

fn cmd_oauth_cancel(args: &[String]) -> Result<String, CodexError> {
    let login_id = args
        .first()
        .ok_or_else(|| CodexError::InvalidState("Usage: oauth cancel <login_id>".into()))?;
    let paths = runtime_paths()?;
    let svc = OAuthService::new(paths);
    svc.cancel_login(login_id)?;
    Ok(json!({"status": "cancelled"}).to_string())
}

fn cmd_account_list() -> Result<String, CodexError> {
    let paths = runtime_paths()?;
    let store = AccountStore::new(paths);
    let accounts = store.list_accounts()?;
    Ok(serde_json::to_string_pretty(&accounts).unwrap_or_default())
}

fn cmd_account_switch(args: &[String]) -> Result<String, CodexError> {
    let account_id = args
        .first()
        .ok_or_else(|| CodexError::InvalidState("Usage: account switch <account_id>".into()))?;
    let paths = runtime_paths()?;
    let store = AccountStore::new(paths);
    store.switch_account_managed(account_id)?;
    Ok(json!({"status": "switched", "account_id": account_id}).to_string())
}

fn cmd_account_add_api_key(args: &[String]) -> Result<String, CodexError> {
    let api_key = args.first().ok_or_else(|| {
        CodexError::InvalidState("Usage: account add-api-key <api_key> [base_url]".into())
    })?;
    let base_url = args.get(1).map(String::as_str);
    let paths = runtime_paths()?;
    let store = AccountStore::new(paths);
    let account = store.upsert_api_key_account(api_key, base_url, None, None, None)?;
    Ok(serde_json::to_string_pretty(&account).unwrap_or_default())
}

fn cmd_account_import_json(args: &[String]) -> Result<String, CodexError> {
    let path = args
        .first()
        .ok_or_else(|| CodexError::InvalidState("Usage: account import-json <path>".into()))?;
    let content = std::fs::read_to_string(path).map_err(CodexError::Io)?;
    let paths = runtime_paths()?;
    let store = AccountStore::new(paths);
    let result = store.import_json_accounts(&content)?;
    Ok(json!({"imported": result.imported, "failed": result.failed}).to_string())
}

fn cmd_account_export(args: &[String]) -> Result<String, CodexError> {
    let paths = runtime_paths()?;
    let store = AccountStore::new(paths);
    let account_ids = args.to_vec();
    let accounts = store.export_accounts(&account_ids)?;
    Ok(serde_json::to_string_pretty(&accounts).unwrap_or_default())
}

fn cmd_quota_refresh() -> Result<String, CodexError> {
    let paths = runtime_paths()?;
    let store = AccountStore::new(paths);
    let current_id = store.get_current_account_id()?;
    match current_id {
        Some(id) => Ok(json!({"status": "quota_refresh_requested", "account_id": id}).to_string()),
        None => Err(CodexError::Quota("No current account".into())),
    }
}

fn cmd_local_access_state() -> Result<String, CodexError> {
    let paths = runtime_paths()?;
    let svc = LocalAccessService::new(paths);
    let collection = svc.load_collection()?;
    let snapshot = svc.get_state_snapshot(&collection, false);
    Ok(serde_json::to_string_pretty(&snapshot).unwrap_or_default())
}

fn cmd_wakeup_status() -> Result<String, CodexError> {
    let scheduler = WakeupScheduler::new();
    let state = scheduler.to_state();
    Ok(serde_json::to_string_pretty(&state).unwrap_or_default())
}

fn handle_command(cmd: &str, args: &[String]) -> Result<String, CodexError> {
    match cmd {
        "oauth" => match args.first().map(|s| s.as_str()) {
            Some("start") => cmd_oauth_start(),
            Some("complete") => cmd_oauth_complete(&args[1..]),
            Some("cancel") => cmd_oauth_cancel(&args[1..]),
            _ => Err(CodexError::InvalidState("Unknown oauth subcommand".into())),
        },
        "account" => match args.first().map(|s| s.as_str()) {
            Some("list") => cmd_account_list(),
            Some("switch") => cmd_account_switch(&args[1..]),
            Some("add-api-key") => cmd_account_add_api_key(&args[1..]),
            Some("import-json") => cmd_account_import_json(&args[1..]),
            Some("export") => cmd_account_export(&args[1..]),
            _ => Err(CodexError::InvalidState(
                "Unknown account subcommand".into(),
            )),
        },
        "quota" => match args.first().map(|s| s.as_str()) {
            Some("refresh") => cmd_quota_refresh(),
            _ => Err(CodexError::InvalidState("Unknown quota subcommand".into())),
        },
        "local-access" => match args.first().map(|s| s.as_str()) {
            Some("state") => cmd_local_access_state(),
            _ => Err(CodexError::InvalidState(
                "Unknown local-access subcommand".into(),
            )),
        },
        "wakeup" => match args.first().map(|s| s.as_str()) {
            Some("status") => cmd_wakeup_status(),
            _ => Err(CodexError::InvalidState("Unknown wakeup subcommand".into())),
        },
        "help" => Ok(HELP_TEXT.to_string()),
        _ => Err(CodexError::InvalidState(format!(
            "Unknown command: {cmd}. Try 'help'"
        ))),
    }
}

const CALLBACK_PORT: u16 = 1455;

const HELP_TEXT: &str = r#"
oauthcodex CLI - Codex OAuth rewrite

Commands:
  oauth start              Start OAuth login flow
  oauth complete <id> <url> Complete OAuth with callback URL
  oauth cancel <id>        Cancel OAuth login
  account list             List all accounts
  account switch <id>      Switch to account
  account add-api-key <key> [base_url]
  account import-json <path>
  account export [account_id...]
  quota refresh            Refresh current account quota
  local-access state       Show local access state
  wakeup status            Show wakeup scheduler status
  help                     Show this help
"#;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        println!("{}", HELP_TEXT);
        return;
    }

    let cmd = &args[0];
    let result = handle_command(cmd, &args[1..]);

    match result {
        Ok(output) => println!("{output}"),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
