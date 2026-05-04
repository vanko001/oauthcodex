import type {
  CodexAccount,
  CodexApiProviderMode,
  CodexFileImportResult,
  CodexOAuthLoginStartResponse,
  CodexQuickConfig,
  CodexQuota,
} from '../types/codex';
import type {
  CodexLocalAccessPortCleanupResult,
  CodexLocalAccessRoutingStrategy,
  CodexLocalAccessState,
  CodexLocalAccessStatsWindow,
  CodexLocalAccessUsageStats,
} from '../types/codexLocalAccess';
import type {
  CodexCliStatus,
  CodexWakeupBatchResult,
  CodexWakeupHistoryItem,
  CodexWakeupModelPreset,
  CodexWakeupOverview,
  CodexWakeupReasoningEffort,
  CodexWakeupState,
  CodexWakeupTask,
} from '../types/codexWakeup';
import type { InstanceProfile, InstanceLaunchMode, InstanceInitMode } from '../types/instance';

export interface CodexBackendAdapter {
  listAccounts(): Promise<CodexAccount[]>;
  getCurrentAccount(): Promise<CodexAccount | null>;
  switchAccount(accountId: string): Promise<CodexAccount>;
  deleteAccount(accountId: string): Promise<void>;
  deleteAccounts(accountIds: string[]): Promise<void>;
  importFromLocal(): Promise<CodexAccount>;
  importFromJson(jsonContent: string): Promise<CodexAccount[]>;
  importFromFiles(filePaths: string[]): Promise<CodexFileImportResult>;
  exportAccounts(accountIds: string[]): Promise<string>;
  refreshQuota(accountId: string): Promise<CodexQuota>;
  refreshAllQuotas(): Promise<number>;
  refreshAccountProfile(accountId: string): Promise<CodexAccount>;
  updateAccountName(accountId: string, name: string): Promise<CodexAccount>;
  updateAccountTags(accountId: string, tags: string[]): Promise<CodexAccount>;
  updateApiKeyCredentials(accountId: string, apiKey: string, apiBaseUrl?: string, apiProviderMode?: CodexApiProviderMode, apiProviderId?: string, apiProviderName?: string): Promise<CodexAccount>;
  startOAuthLogin(): Promise<CodexOAuthLoginStartResponse>;
  completeOAuthLogin(loginId: string): Promise<CodexAccount>;
  cancelOAuthLogin(loginId?: string): Promise<void>;
  submitOAuthCallbackUrl(loginId: string, callbackUrl: string): Promise<void>;
  addAccountWithToken(idToken: string, accessToken: string, refreshToken?: string): Promise<CodexAccount>;
  addAccountWithApiKey(apiKey: string, apiBaseUrl?: string, apiProviderMode?: CodexApiProviderMode, apiProviderId?: string, apiProviderName?: string): Promise<CodexAccount>;
  isOAuthPortInUse(): Promise<boolean>;
  closeOAuthPort(): Promise<number>;
  getConfigTomlPath(): Promise<string>;
  openConfigToml(): Promise<void>;
  getQuickConfig(): Promise<CodexQuickConfig>;
  saveQuickConfig(modelContextWindow?: number, autoCompactTokenLimit?: number): Promise<CodexQuickConfig>;

  getLocalAccessState(): Promise<CodexLocalAccessState>;
  saveLocalAccessAccounts(accountIds: string[], restrictFreeAccounts: boolean): Promise<CodexLocalAccessState>;
  removeLocalAccessAccount(accountId: string): Promise<CodexLocalAccessState>;
  rotateLocalAccessApiKey(): Promise<CodexLocalAccessState>;
  clearLocalAccessStats(): Promise<CodexLocalAccessState>;
  prepareLocalAccessForRestart(): Promise<CodexLocalAccessState>;
  killLocalAccessPort(): Promise<CodexLocalAccessPortCleanupResult>;
  updateLocalAccessPort(port: number): Promise<CodexLocalAccessState>;
  updateLocalAccessRoutingStrategy(strategy: CodexLocalAccessRoutingStrategy): Promise<CodexLocalAccessState>;
  setLocalAccessEnabled(enabled: boolean): Promise<CodexLocalAccessState>;
  activateLocalAccess(): Promise<CodexLocalAccessState>;

  getInstanceDefaults(): Promise<{ rootDir: string; defaultUserDataDir: string }>;
  listInstances(): Promise<InstanceProfile[]>;
  createInstance(payload: { name: string; userDataDir: string; workingDir?: string | null; extraArgs?: string; bindAccountId?: string | null; launchMode?: InstanceLaunchMode; copySourceInstanceId: string; initMode?: InstanceInitMode }): Promise<InstanceProfile>;
  updateInstance(payload: { instanceId: string; name?: string; workingDir?: string | null; extraArgs?: string; bindAccountId?: string | null; followLocalAccount?: boolean; launchMode?: InstanceLaunchMode }): Promise<InstanceProfile>;
  deleteInstance(instanceId: string): Promise<void>;
  startInstance(instanceId: string): Promise<InstanceProfile>;
  stopInstance(instanceId: string): Promise<InstanceProfile>;
  closeAllInstances(): Promise<void>;
  openInstanceWindow(instanceId: string): Promise<void>;
  getInstanceQuickConfig(instanceId: string): Promise<CodexQuickConfig>;
  saveInstanceQuickConfig(instanceId: string, modelContextWindow?: number, autoCompactTokenLimit?: number): Promise<CodexQuickConfig>;
  openInstanceConfigToml(instanceId: string): Promise<void>;
  getInstanceLaunchCommand(instanceId: string): Promise<{ instanceId: string; userDataDir: string; launchCommand: string }>;
  executeInstanceLaunchCommand(instanceId: string, terminal?: string): Promise<string>;
  syncThreadsAcrossInstances(): Promise<{ message: string }>;
  repairSessionVisibilityAcrossInstances(): Promise<{ message: string }>;
  listSessionsAcrossInstances(): Promise<any[]>;
  getSessionTokenStatsAcrossInstances(sessionIds: string[]): Promise<any[]>;
  moveSessionsToTrashAcrossInstances(sessionIds: string[]): Promise<{ message: string }>;
  listTrashedSessionsAcrossInstances(): Promise<any[]>;
  restoreSessionsFromTrashAcrossInstances(sessionIds: string[]): Promise<{ message: string }>;

  getWakeupCliStatus(): Promise<CodexCliStatus>;
  updateWakeupRuntimeConfig(codexCliPath?: string, nodePath?: string): Promise<CodexCliStatus>;
  getWakeupOverview(): Promise<CodexWakeupOverview>;
  saveWakeupState(enabled: boolean, tasks: CodexWakeupTask[], modelPresets: CodexWakeupModelPreset[], modelPresetMigrations?: string[]): Promise<CodexWakeupState>;
  loadWakeupHistory(): Promise<CodexWakeupHistoryItem[]>;
  clearWakeupHistory(): Promise<void>;
  testWakeup(accountIds: string[], runId: string, prompt?: string, model?: string, modelDisplayName?: string, modelReasoningEffort?: CodexWakeupReasoningEffort, cancelScopeId?: string): Promise<CodexWakeupBatchResult>;
  runWakeupTask(taskId: string, runId: string): Promise<CodexWakeupBatchResult>;
  cancelWakeupScope(cancelScopeId: string): Promise<void>;
  releaseWakeupScope(cancelScopeId: string): Promise<void>;

  getGeneralConfig(): Promise<Record<string, unknown>>;
  saveGeneralConfig(config: Record<string, unknown>): Promise<void>;

  listCodexAccountGroups(): Promise<{ id: string; name: string; accountIds: string[]; sortOrder?: number }[]>;
  createCodexGroup(name: string): Promise<{ id: string; name: string; accountIds: string[]; sortOrder?: number }>;
  renameCodexGroup(id: string, name: string): Promise<void>;
  deleteCodexGroup(id: string): Promise<void>;
  assignAccountsToCodexGroup(groupId: string, accountIds: string[]): Promise<void>;
  removeAccountsFromCodexGroup(groupId: string, accountIds: string[]): Promise<void>;

  listCodexModelProviders(): Promise<{ id: string; name: string; baseUrl: string; website?: string; apiKeyUrl?: string; apiKeys: { id: string; name?: string; apiKey: string }[] }[]>;
  createCodexModelProvider(payload: { name: string; baseUrl: string; website?: string; apiKeyUrl?: string; initialApiKey?: string; initialApiKeyName?: string }): Promise<{ id: string; name: string; baseUrl: string; website?: string; apiKeyUrl?: string; apiKeys: { id: string; name?: string; apiKey: string }[] }>;
  updateCodexModelProvider(id: string, payload: { name?: string; baseUrl?: string; website?: string; apiKeyUrl?: string }): Promise<void>;
  deleteCodexModelProvider(id: string): Promise<void>;
  addApiKeyToCodexModelProvider(providerId: string, apiKey: string, name?: string): Promise<void>;
  removeApiKeyFromCodexModelProvider(providerId: string, apiKeyId: string): Promise<void>;
}

type JsonRecord = Record<string, unknown>;
type InvokeFn = <T = unknown>(command: string, args?: Record<string, unknown>) => Promise<T>;

declare global {
  interface Window {
    __OAUTHCODEX_BACKEND__?: CodexBackendAdapter;
    __TAURI__?: { core?: { invoke?: InvokeFn } };
    __TAURI_INTERNALS__?: { invoke?: InvokeFn };
  }
}

function isRecord(value: unknown): value is JsonRecord {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function readKey(record: JsonRecord | null | undefined, ...keys: string[]): unknown {
  if (!record) return undefined;
  for (const key of keys) {
    if (Object.prototype.hasOwnProperty.call(record, key)) return record[key];
  }
  return undefined;
}

function asString(value: unknown, fallback = ''): string {
  if (typeof value === 'string') return value;
  if (typeof value === 'number' || typeof value === 'boolean') return String(value);
  return fallback;
}

function optionalString(value: unknown): string | undefined {
  const text = asString(value).trim();
  return text || undefined;
}

function asBool(value: unknown, fallback = false): boolean {
  if (typeof value === 'boolean') return value;
  if (typeof value === 'number') return value !== 0;
  if (typeof value === 'string') {
    const normalized = value.trim().toLowerCase();
    if (normalized === 'true' || normalized === '1') return true;
    if (normalized === 'false' || normalized === '0') return false;
  }
  return fallback;
}

function asNumber(value: unknown, fallback = 0): number {
  if (typeof value === 'number' && Number.isFinite(value)) return value;
  if (typeof value === 'string' && value.trim()) {
    const numeric = Number(value);
    if (Number.isFinite(numeric)) return numeric;
  }
  return fallback;
}

function optionalNumber(value: unknown): number | undefined {
  if (typeof value === 'number' && Number.isFinite(value)) return value;
  if (typeof value === 'string' && value.trim()) {
    const numeric = Number(value);
    if (Number.isFinite(numeric)) return numeric;
  }
  return undefined;
}

function epochSeconds(value: unknown, fallback = 0): number {
  if (typeof value === 'number' && Number.isFinite(value)) {
    return value > 10_000_000_000 ? Math.floor(value / 1000) : Math.floor(value);
  }
  if (typeof value === 'string' && value.trim()) {
    const numeric = Number(value);
    if (Number.isFinite(numeric)) return epochSeconds(numeric, fallback);
    const parsed = Date.parse(value);
    if (Number.isFinite(parsed)) return Math.floor(parsed / 1000);
  }
  return fallback;
}

function stringArray(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return value.map((item) => optionalString(item)).filter((item): item is string => Boolean(item));
}

function normalizeApiProviderMode(value: unknown): CodexApiProviderMode | undefined {
  const mode = optionalString(value)?.toLowerCase();
  if (!mode) return undefined;
  if (mode === 'openai_builtin') return 'openai';
  if (mode === 'openai' || mode === 'custom' || mode === 'azure') return mode;
  return undefined;
}

function normalizeLaunchMode(value: unknown): InstanceLaunchMode {
  const mode = optionalString(value)?.toLowerCase();
  if (mode === 'manual' || mode === 'cli' || mode === 'auto' || mode === 'app') return mode;
  return 'auto';
}

export function normalizeCodexQuota(raw: unknown): CodexQuota | undefined {
  if (!isRecord(raw)) return undefined;
  const windows = Array.isArray(raw.windows) ? raw.windows.filter(isRecord) : [];
  const primary = windows.find((item) => readKey(item, 'type', 'window_type') === 'primary' || readKey(item, 'label') === '5h') ?? windows[0];
  const secondary = windows.find((item) => readKey(item, 'type', 'window_type') === 'secondary' || readKey(item, 'label') === 'Weekly') ?? windows[1];
  return {
    hourly_percentage: asNumber(readKey(raw, 'hourly_percentage', 'hourlyPercentage', 'primary_percentage') ?? readKey(primary, 'percentage'), 0),
    hourly_reset_time: epochSeconds(readKey(raw, 'hourly_reset_time', 'hourlyResetTime') ?? readKey(primary, 'reset_at', 'resetAt'), 0) || undefined,
    hourly_window_minutes: optionalNumber(readKey(raw, 'hourly_window_minutes', 'hourlyWindowMinutes')),
    hourly_window_present: readKey(raw, 'hourly_window_present', 'hourlyWindowPresent') as boolean | undefined,
    weekly_percentage: asNumber(readKey(raw, 'weekly_percentage', 'weeklyPercentage', 'secondary_percentage') ?? readKey(secondary, 'percentage'), 0),
    weekly_reset_time: epochSeconds(readKey(raw, 'weekly_reset_time', 'weeklyResetTime') ?? readKey(secondary, 'reset_at', 'resetAt'), 0) || undefined,
    weekly_window_minutes: optionalNumber(readKey(raw, 'weekly_window_minutes', 'weeklyWindowMinutes')),
    weekly_window_present: readKey(raw, 'weekly_window_present', 'weeklyWindowPresent') as boolean | undefined,
    raw_data: readKey(raw, 'raw_data', 'rawData') ?? raw,
    code_review_quota: isRecord(raw.code_review_quota) ? raw.code_review_quota as never : isRecord(raw.codeReviewQuota) ? raw.codeReviewQuota as never : undefined,
  };
}

export function normalizeCodexAccount(raw: unknown): CodexAccount {
  const record = isRecord(raw) ? raw : {};
  const authMode = optionalString(readKey(record, 'auth_mode', 'authMode')) ?? 'oauth';
  const displayName = optionalString(readKey(record, 'display_name', 'displayName', 'account_name', 'accountName'));
  const email = optionalString(readKey(record, 'email')) ?? displayName ?? optionalString(readKey(record, 'provider_name', 'api_provider_name')) ?? (authMode === 'apikey' ? 'API Key Account' : 'Unknown Codex Account');
  const tokensRecord = isRecord(record.tokens) ? record.tokens : {};
  const createdAt = epochSeconds(readKey(record, 'created_at', 'createdAt'), Math.floor(Date.now() / 1000));
  const lastUsed = epochSeconds(readKey(record, 'last_used', 'lastUsed'), createdAt);
  return {
    ...(record as Partial<CodexAccount>),
    id: asString(readKey(record, 'id'), `acct_${Date.now()}`),
    provider: optionalString(readKey(record, 'provider')) ?? 'codex',
    email,
    auth_mode: authMode,
    display_name: displayName ?? email,
    openai_api_key: optionalString(readKey(record, 'openai_api_key', 'openaiApiKey', 'api_key', 'apiKey')),
    api_base_url: optionalString(readKey(record, 'api_base_url', 'apiBaseUrl', 'base_url', 'baseUrl')),
    api_provider_mode: normalizeApiProviderMode(readKey(record, 'api_provider_mode', 'apiProviderMode')),
    api_provider_id: optionalString(readKey(record, 'api_provider_id', 'apiProviderId', 'provider_id', 'providerId')),
    api_provider_name: optionalString(readKey(record, 'api_provider_name', 'apiProviderName', 'provider_name', 'providerName')),
    user_id: optionalString(readKey(record, 'user_id', 'userId')),
    plan_type: optionalString(readKey(record, 'plan_type', 'planType')),
    account_id: optionalString(readKey(record, 'account_id', 'accountId')),
    organization_id: optionalString(readKey(record, 'organization_id', 'organizationId')),
    account_name: optionalString(readKey(record, 'account_name', 'accountName')) ?? displayName,
    tokens: {
      id_token: asString(readKey(tokensRecord, 'id_token', 'idToken')),
      access_token: asString(readKey(tokensRecord, 'access_token', 'accessToken')),
      refresh_token: optionalString(readKey(tokensRecord, 'refresh_token', 'refreshToken')),
    },
    tags: stringArray(readKey(record, 'tags')),
    quota: normalizeCodexQuota(readKey(record, 'quota')) as CodexQuota | undefined,
    quota_error: isRecord(readKey(record, 'quota_error', 'quotaError')) ? readKey(record, 'quota_error', 'quotaError') as never : undefined,
    created_at: createdAt,
    last_used: lastUsed,
  };
}

export function normalizeCodexAccounts(raw: unknown): CodexAccount[] {
  const list = Array.isArray(raw) ? raw : isRecord(raw) && Array.isArray(raw.accounts) ? raw.accounts : [];
  return list.map(normalizeCodexAccount);
}

export function normalizeOAuthLoginStartResponse(raw: unknown): CodexOAuthLoginStartResponse {
  const record = isRecord(raw) ? raw : {};
  return {
    loginId: asString(readKey(record, 'loginId', 'login_id')),
    authUrl: asString(readKey(record, 'authUrl', 'auth_url')),
  };
}

export function normalizeQuickConfig(raw: unknown): CodexQuickConfig {
  const record = isRecord(raw) ? raw : {};
  const modelContextWindow = asNumber(readKey(record, 'model_context_window', 'modelContextWindow', 'detected_model_context_window'), 1_000_000);
  const autoCompact = asNumber(readKey(record, 'model_auto_compact_token_limit', 'modelAutoCompactTokenLimit', 'auto_compact_token_limit', 'autoCompactTokenLimit'), 900_000);
  return {
    context_window_1m: modelContextWindow >= 1_000_000,
    auto_compact_token_limit: autoCompact,
    detected_model_context_window: modelContextWindow,
    detected_auto_compact_token_limit: autoCompact,
  };
}

function emptyUsageStats(): CodexLocalAccessUsageStats {
  return {
    requestCount: 0,
    successCount: 0,
    failureCount: 0,
    totalLatencyMs: 0,
    inputTokens: 0,
    outputTokens: 0,
    totalTokens: 0,
    cachedTokens: 0,
    reasoningTokens: 0,
  };
}

function normalizeStatsWindow(raw: unknown): CodexLocalAccessStatsWindow {
  const record = isRecord(raw) ? raw : {};
  const totalsRecord = isRecord(record.totals) ? record.totals : record;
  const inputTokens = asNumber(readKey(totalsRecord, 'inputTokens', 'input_tokens', 'tokens_in'), 0);
  const outputTokens = asNumber(readKey(totalsRecord, 'outputTokens', 'output_tokens', 'tokens_out'), 0);
  const totals: CodexLocalAccessUsageStats = {
    ...emptyUsageStats(),
    requestCount: asNumber(readKey(totalsRecord, 'requestCount', 'request_count', 'requests'), 0),
    successCount: asNumber(readKey(totalsRecord, 'successCount', 'success_count', 'successes'), 0),
    failureCount: asNumber(readKey(totalsRecord, 'failureCount', 'failure_count', 'failures'), 0),
    totalLatencyMs: asNumber(readKey(totalsRecord, 'totalLatencyMs', 'total_latency_ms', 'latency_ms_sum'), 0),
    inputTokens,
    outputTokens,
    totalTokens: asNumber(readKey(totalsRecord, 'totalTokens', 'total_tokens'), inputTokens + outputTokens),
    cachedTokens: asNumber(readKey(totalsRecord, 'cachedTokens', 'cached_tokens'), 0),
    reasoningTokens: asNumber(readKey(totalsRecord, 'reasoningTokens', 'reasoning_tokens'), 0),
  };
  return {
    since: epochSeconds(readKey(record, 'since'), 0),
    updatedAt: epochSeconds(readKey(record, 'updatedAt', 'updated_at'), 0),
    totals,
    accounts: Array.isArray(record.accounts) ? record.accounts.filter(isRecord).map((item) => ({
      accountId: asString(readKey(item, 'accountId', 'account_id')),
      email: asString(readKey(item, 'email')),
      usage: normalizeStatsWindow(item.usage).totals,
      updatedAt: epochSeconds(readKey(item, 'updatedAt', 'updated_at'), 0),
    })) : [],
  };
}

function normalizeLocalAccessCollection(raw: unknown, parent?: JsonRecord): CodexLocalAccessState['collection'] {
  const record = isRecord(raw) ? raw : parent && (readKey(parent, 'local_api_key', 'localApiKey') || readKey(parent, 'port') !== undefined) ? parent : null;
  if (!record) return null;
  return {
    enabled: asBool(readKey(record, 'enabled'), false),
    port: asNumber(readKey(record, 'port'), 0),
    apiKey: asString(readKey(record, 'apiKey', 'api_key', 'local_api_key', 'localApiKey')),
    routingStrategy: (optionalString(readKey(record, 'routingStrategy', 'routing_strategy')) ?? 'auto') as CodexLocalAccessRoutingStrategy,
    restrictFreeAccounts: asBool(readKey(record, 'restrictFreeAccounts', 'restrict_free_accounts'), true),
    accountIds: stringArray(readKey(record, 'accountIds', 'account_ids', 'accounts')),
    createdAt: epochSeconds(readKey(record, 'createdAt', 'created_at'), 0),
    updatedAt: epochSeconds(readKey(record, 'updatedAt', 'updated_at'), 0),
  };
}

export function normalizeCodexLocalAccessState(raw: unknown): CodexLocalAccessState {
  const record = isRecord(raw) ? raw : {};
  const statsRecord = isRecord(record.stats) ? record.stats : {};
  const daily = normalizeStatsWindow(readKey(statsRecord, 'daily'));
  const weekly = normalizeStatsWindow(readKey(statsRecord, 'weekly'));
  const monthly = normalizeStatsWindow(readKey(statsRecord, 'monthly'));
  const totals = isRecord(statsRecord.totals) ? normalizeStatsWindow(statsRecord).totals : monthly.totals.requestCount ? monthly.totals : daily.totals;
  return {
    collection: normalizeLocalAccessCollection(readKey(record, 'collection'), record),
    running: asBool(readKey(record, 'running'), false),
    apiPortUrl: optionalString(readKey(record, 'apiPortUrl', 'api_port_url')) ?? null,
    baseUrl: optionalString(readKey(record, 'baseUrl', 'base_url')) ?? null,
    modelIds: stringArray(readKey(record, 'modelIds', 'model_ids')),
    lastError: optionalString(readKey(record, 'lastError', 'last_error')) ?? null,
    memberCount: asNumber(readKey(record, 'memberCount', 'member_count', 'account_count'), 0),
    stats: {
      since: epochSeconds(readKey(statsRecord, 'since'), 0),
      updatedAt: epochSeconds(readKey(statsRecord, 'updatedAt', 'updated_at'), 0),
      totals,
      accounts: Array.isArray(statsRecord.accounts) ? normalizeStatsWindow(statsRecord).accounts : [],
      daily,
      weekly,
      monthly,
    },
  };
}

export function normalizeCodexGroup(raw: unknown): { id: string; name: string; accountIds: string[]; sortOrder?: number } {
  const record = isRecord(raw) ? raw : {};
  return {
    id: asString(readKey(record, 'id'), `cgrp_${Date.now()}`),
    name: asString(readKey(record, 'name'), 'Group'),
    accountIds: stringArray(readKey(record, 'accountIds', 'account_ids')),
    sortOrder: asNumber(readKey(record, 'sortOrder', 'sort_order'), 0),
  };
}

export function normalizeCodexModelProvider(raw: unknown): { id: string; name: string; baseUrl: string; website?: string; apiKeyUrl?: string; apiKeys: { id: string; name?: string; apiKey: string }[] } {
  const record = isRecord(raw) ? raw : {};
  const rawKeys = readKey(record, 'apiKeys', 'api_keys');
  const apiKeys = Array.isArray(rawKeys) ? rawKeys.filter(isRecord).map((item) => ({
    id: asString(readKey(item, 'id'), `cmk_${Date.now()}`),
    name: optionalString(readKey(item, 'name')),
    apiKey: asString(readKey(item, 'apiKey', 'api_key', 'key')),
  })).filter((item) => item.apiKey) : [];
  return {
    id: asString(readKey(record, 'id'), `cmp_${Date.now()}`),
    name: asString(readKey(record, 'name'), 'Provider'),
    baseUrl: asString(readKey(record, 'baseUrl', 'base_url')),
    website: optionalString(readKey(record, 'website')),
    apiKeyUrl: optionalString(readKey(record, 'apiKeyUrl', 'api_key_url')),
    apiKeys,
  };
}

export function normalizeCodexInstance(raw: unknown): InstanceProfile {
  const record = isRecord(raw) ? raw : {};
  const createdAt = epochSeconds(readKey(record, 'createdAt', 'created_at'), 0);
  const lastPid = asNumber(readKey(record, 'lastPid', 'last_pid'), 0) || null;
  const extraArgsRaw = readKey(record, 'extraArgs', 'extra_args');
  return {
    id: asString(readKey(record, 'id'), `inst_${Date.now()}`),
    name: asString(readKey(record, 'name'), 'Default'),
    userDataDir: asString(readKey(record, 'userDataDir', 'user_data_dir')),
    workingDir: optionalString(readKey(record, 'workingDir', 'working_dir')) ?? null,
    extraArgs: Array.isArray(extraArgsRaw) ? extraArgsRaw.map(String).join(' ') : asString(extraArgsRaw),
    bindAccountId: optionalString(readKey(record, 'bindAccountId', 'bind_account_id')) ?? null,
    launchMode: normalizeLaunchMode(readKey(record, 'launchMode', 'launch_mode')),
    createdAt,
    lastLaunchedAt: epochSeconds(readKey(record, 'lastLaunchedAt', 'last_launched_at', 'updatedAt', 'updated_at'), 0) || null,
    lastPid,
    running: asBool(readKey(record, 'running'), Boolean(lastPid)),
    initialized: asBool(readKey(record, 'initialized'), false),
    isDefault: asBool(readKey(record, 'isDefault', 'is_default'), false),
    followLocalAccount: asBool(readKey(record, 'followLocalAccount', 'follow_local_account'), false),
  };
}

function parseJsonArray(raw: unknown): unknown[] {
  const parsed = typeof raw === 'string' ? (raw.trim() ? JSON.parse(raw) : []) : raw;
  if (Array.isArray(parsed)) return parsed;
  if (isRecord(parsed) && Array.isArray(parsed.groups)) return parsed.groups;
  if (isRecord(parsed) && Array.isArray(parsed.providers)) return parsed.providers;
  if (isRecord(parsed) && Array.isArray(parsed.instances)) return parsed.instances;
  return [];
}

function normalizeBaseUrl(value: string): string {
  try {
    const parsed = new URL(value.trim());
    return `${parsed.origin}${parsed.pathname}`.replace(/\/+$/, '').toLowerCase();
  } catch {
    return value.trim().replace(/\/+$/, '').toLowerCase();
  }
}

class UnavailableBackendAdapter implements CodexBackendAdapter {
  private fail(): never {
    throw new Error('No oauthcodex backend adapter is available. Provide window.__OAUTHCODEX_BACKEND__ or run inside the Tauri shell.');
  }
  listAccounts(): Promise<CodexAccount[]> { this.fail(); }
  getCurrentAccount(): Promise<CodexAccount | null> { this.fail(); }
  switchAccount(_accountId: string): Promise<CodexAccount> { this.fail(); }
  deleteAccount(_accountId: string): Promise<void> { this.fail(); }
  deleteAccounts(_accountIds: string[]): Promise<void> { this.fail(); }
  importFromLocal(): Promise<CodexAccount> { this.fail(); }
  importFromJson(_jsonContent: string): Promise<CodexAccount[]> { this.fail(); }
  importFromFiles(_filePaths: string[]): Promise<CodexFileImportResult> { this.fail(); }
  exportAccounts(_accountIds: string[]): Promise<string> { this.fail(); }
  refreshQuota(_accountId: string): Promise<CodexQuota> { this.fail(); }
  refreshAllQuotas(): Promise<number> { this.fail(); }
  refreshAccountProfile(_accountId: string): Promise<CodexAccount> { this.fail(); }
  updateAccountName(_accountId: string, _name: string): Promise<CodexAccount> { this.fail(); }
  updateAccountTags(_accountId: string, _tags: string[]): Promise<CodexAccount> { this.fail(); }
  updateApiKeyCredentials(_accountId: string, _apiKey: string, _apiBaseUrl?: string, _apiProviderMode?: CodexApiProviderMode, _apiProviderId?: string, _apiProviderName?: string): Promise<CodexAccount> { this.fail(); }
  startOAuthLogin(): Promise<CodexOAuthLoginStartResponse> { this.fail(); }
  completeOAuthLogin(_loginId: string): Promise<CodexAccount> { this.fail(); }
  cancelOAuthLogin(_loginId?: string): Promise<void> { this.fail(); }
  submitOAuthCallbackUrl(_loginId: string, _callbackUrl: string): Promise<void> { this.fail(); }
  addAccountWithToken(_idToken: string, _accessToken: string, _refreshToken?: string): Promise<CodexAccount> { this.fail(); }
  addAccountWithApiKey(_apiKey: string, _apiBaseUrl?: string, _apiProviderMode?: CodexApiProviderMode, _apiProviderId?: string, _apiProviderName?: string): Promise<CodexAccount> { this.fail(); }
  isOAuthPortInUse(): Promise<boolean> { this.fail(); }
  closeOAuthPort(): Promise<number> { this.fail(); }
  getConfigTomlPath(): Promise<string> { this.fail(); }
  openConfigToml(): Promise<void> { this.fail(); }
  getQuickConfig(): Promise<CodexQuickConfig> { this.fail(); }
  saveQuickConfig(_modelContextWindow?: number, _autoCompactTokenLimit?: number): Promise<CodexQuickConfig> { this.fail(); }
  getLocalAccessState(): Promise<CodexLocalAccessState> { this.fail(); }
  saveLocalAccessAccounts(_accountIds: string[], _restrictFreeAccounts: boolean): Promise<CodexLocalAccessState> { this.fail(); }
  removeLocalAccessAccount(_accountId: string): Promise<CodexLocalAccessState> { this.fail(); }
  rotateLocalAccessApiKey(): Promise<CodexLocalAccessState> { this.fail(); }
  clearLocalAccessStats(): Promise<CodexLocalAccessState> { this.fail(); }
  prepareLocalAccessForRestart(): Promise<CodexLocalAccessState> { this.fail(); }
  killLocalAccessPort(): Promise<CodexLocalAccessPortCleanupResult> { this.fail(); }
  updateLocalAccessPort(_port: number): Promise<CodexLocalAccessState> { this.fail(); }
  updateLocalAccessRoutingStrategy(_strategy: CodexLocalAccessRoutingStrategy): Promise<CodexLocalAccessState> { this.fail(); }
  setLocalAccessEnabled(_enabled: boolean): Promise<CodexLocalAccessState> { this.fail(); }
  activateLocalAccess(): Promise<CodexLocalAccessState> { this.fail(); }
  getInstanceDefaults(): Promise<{ rootDir: string; defaultUserDataDir: string }> { this.fail(); }
  listInstances(): Promise<InstanceProfile[]> { this.fail(); }
  createInstance(_payload: { name: string; userDataDir: string; workingDir?: string | null; extraArgs?: string; bindAccountId?: string | null; launchMode?: InstanceLaunchMode; copySourceInstanceId: string; initMode?: InstanceInitMode }): Promise<InstanceProfile> { this.fail(); }
  updateInstance(_payload: { instanceId: string; name?: string; workingDir?: string | null; extraArgs?: string; bindAccountId?: string | null; followLocalAccount?: boolean; launchMode?: InstanceLaunchMode }): Promise<InstanceProfile> { this.fail(); }
  deleteInstance(_instanceId: string): Promise<void> { this.fail(); }
  startInstance(_instanceId: string): Promise<InstanceProfile> { this.fail(); }
  stopInstance(_instanceId: string): Promise<InstanceProfile> { this.fail(); }
  closeAllInstances(): Promise<void> { this.fail(); }
  openInstanceWindow(_instanceId: string): Promise<void> { this.fail(); }
  getInstanceQuickConfig(_instanceId: string): Promise<CodexQuickConfig> { this.fail(); }
  saveInstanceQuickConfig(_instanceId: string, _modelContextWindow?: number, _autoCompactTokenLimit?: number): Promise<CodexQuickConfig> { this.fail(); }
  openInstanceConfigToml(_instanceId: string): Promise<void> { this.fail(); }
  getInstanceLaunchCommand(_instanceId: string): Promise<{ instanceId: string; userDataDir: string; launchCommand: string }> { this.fail(); }
  executeInstanceLaunchCommand(_instanceId: string, _terminal?: string): Promise<string> { this.fail(); }
  syncThreadsAcrossInstances(): Promise<{ message: string }> { this.fail(); }
  repairSessionVisibilityAcrossInstances(): Promise<{ message: string }> { this.fail(); }
  listSessionsAcrossInstances(): Promise<any[]> { this.fail(); }
  getSessionTokenStatsAcrossInstances(_sessionIds: string[]): Promise<any[]> { this.fail(); }
  moveSessionsToTrashAcrossInstances(_sessionIds: string[]): Promise<{ message: string }> { this.fail(); }
  listTrashedSessionsAcrossInstances(): Promise<any[]> { this.fail(); }
  restoreSessionsFromTrashAcrossInstances(_sessionIds: string[]): Promise<{ message: string }> { this.fail(); }
  getWakeupCliStatus(): Promise<CodexCliStatus> { this.fail(); }
  updateWakeupRuntimeConfig(_codexCliPath?: string, _nodePath?: string): Promise<CodexCliStatus> { this.fail(); }
  getWakeupOverview(): Promise<CodexWakeupOverview> { this.fail(); }
  saveWakeupState(_enabled: boolean, _tasks: CodexWakeupTask[], _modelPresets: CodexWakeupModelPreset[], _modelPresetMigrations?: string[]): Promise<CodexWakeupState> { this.fail(); }
  loadWakeupHistory(): Promise<CodexWakeupHistoryItem[]> { this.fail(); }
  clearWakeupHistory(): Promise<void> { this.fail(); }
  testWakeup(_accountIds: string[], _runId: string, _prompt?: string, _model?: string, _modelDisplayName?: string, _modelReasoningEffort?: CodexWakeupReasoningEffort, _cancelScopeId?: string): Promise<CodexWakeupBatchResult> { this.fail(); }
  runWakeupTask(_taskId: string, _runId: string): Promise<CodexWakeupBatchResult> { this.fail(); }
  cancelWakeupScope(_cancelScopeId: string): Promise<void> { this.fail(); }
  releaseWakeupScope(_cancelScopeId: string): Promise<void> { this.fail(); }
  getGeneralConfig(): Promise<Record<string, unknown>> { this.fail(); }
  saveGeneralConfig(_config: Record<string, unknown>): Promise<void> { this.fail(); }
  listCodexAccountGroups(): Promise<{ id: string; name: string; accountIds: string[] }[]> { this.fail(); }
  createCodexGroup(_name: string): Promise<{ id: string; name: string; accountIds: string[] }> { this.fail(); }
  renameCodexGroup(_id: string, _name: string): Promise<void> { this.fail(); }
  deleteCodexGroup(_id: string): Promise<void> { this.fail(); }
  assignAccountsToCodexGroup(_groupId: string, _accountIds: string[]): Promise<void> { this.fail(); }
  removeAccountsFromCodexGroup(_groupId: string, _accountIds: string[]): Promise<void> { this.fail(); }
  listCodexModelProviders(): Promise<{ id: string; name: string; baseUrl: string; website?: string; apiKeyUrl?: string; apiKeys: { id: string; name?: string; apiKey: string }[] }[]> { this.fail(); }
  createCodexModelProvider(_payload: { name: string; baseUrl: string; website?: string; apiKeyUrl?: string; initialApiKey?: string; initialApiKeyName?: string }): Promise<{ id: string; name: string; baseUrl: string; website?: string; apiKeyUrl?: string; apiKeys: { id: string; name?: string; apiKey: string }[] }> { this.fail(); }
  updateCodexModelProvider(_id: string, _payload: { name?: string; baseUrl?: string; website?: string; apiKeyUrl?: string }): Promise<void> { this.fail(); }
  deleteCodexModelProvider(_id: string): Promise<void> { this.fail(); }
  addApiKeyToCodexModelProvider(_providerId: string, _apiKey: string, _name?: string): Promise<void> { this.fail(); }
  removeApiKeyFromCodexModelProvider(_providerId: string, _apiKeyId: string): Promise<void> { this.fail(); }
}

class TauriBackendAdapter implements CodexBackendAdapter {
  constructor(private readonly invoke: InvokeFn) {}

  async listAccounts() { return normalizeCodexAccounts(await this.invoke('list_codex_accounts')); }
  async getCurrentAccount() {
    const account = await this.invoke('get_current_codex_account');
    return account ? normalizeCodexAccount(account) : null;
  }
  async switchAccount(accountId: string) { return normalizeCodexAccount(await this.invoke('switch_codex_account', { accountId })); }
  async deleteAccount(accountId: string) { await this.invoke('delete_codex_account', { accountId }); }
  async deleteAccounts(accountIds: string[]) { await this.invoke('delete_codex_accounts', { accountIds }); }
  async importFromLocal() { return normalizeCodexAccount(await this.invoke('import_codex_from_local')); }
  async importFromJson(jsonContent: string) { return normalizeCodexAccounts(await this.invoke('import_codex_from_json', { jsonContent })); }
  async importFromFiles(filePaths: string[]) {
    const result = await this.invoke('import_codex_from_files', { filePaths });
    const record = isRecord(result) ? result : {};
    return {
      imported: normalizeCodexAccounts(record.imported),
      failed: Array.isArray(record.failed) ? record.failed as { email: string; error: string }[] : [],
    };
  }
  async exportAccounts(accountIds: string[]) { return this.invoke<string>('export_codex_accounts', { accountIds }); }
  async refreshQuota(accountId: string) { return normalizeCodexQuota(await this.invoke('refresh_codex_quota', { accountId })) ?? { hourly_percentage: 0, weekly_percentage: 0 }; }
  async refreshAllQuotas() { return this.invoke<number>('refresh_all_codex_quotas'); }
  async refreshAccountProfile(accountId: string) { return normalizeCodexAccount(await this.invoke('refresh_codex_account_profile', { accountId })); }
  async updateAccountName(accountId: string, name: string) { return normalizeCodexAccount(await this.invoke('update_codex_account_name', { accountId, name })); }
  async updateAccountTags(accountId: string, tags: string[]) { return normalizeCodexAccount(await this.invoke('update_codex_account_tags', { accountId, tags })); }
  async updateApiKeyCredentials(accountId: string, apiKey: string, apiBaseUrl?: string, apiProviderMode?: CodexApiProviderMode, apiProviderId?: string, apiProviderName?: string) {
    return normalizeCodexAccount(await this.invoke('update_codex_api_key_credentials', { accountId, apiKey, apiBaseUrl: apiBaseUrl ?? null, apiProviderMode: apiProviderMode ?? null, apiProviderId: apiProviderId ?? null, apiProviderName: apiProviderName ?? null }));
  }
  async startOAuthLogin() { return normalizeOAuthLoginStartResponse(await this.invoke('codex_oauth_login_start')); }
  async completeOAuthLogin(loginId: string) { return normalizeCodexAccount(await this.invoke('codex_oauth_login_completed', { loginId })); }
  async cancelOAuthLogin(loginId?: string) { await this.invoke('codex_oauth_login_cancel', { loginId: loginId ?? null }); }
  async submitOAuthCallbackUrl(loginId: string, callbackUrl: string) { await this.invoke('codex_oauth_submit_callback_url', { loginId, callbackUrl }); }
  async addAccountWithToken(idToken: string, accessToken: string, refreshToken?: string) { return normalizeCodexAccount(await this.invoke('add_codex_account_with_token', { idToken, accessToken, refreshToken: refreshToken ?? null })); }
  async addAccountWithApiKey(apiKey: string, apiBaseUrl?: string, apiProviderMode?: CodexApiProviderMode, apiProviderId?: string, apiProviderName?: string) {
    return normalizeCodexAccount(await this.invoke('add_codex_account_with_api_key', { apiKey, apiBaseUrl: apiBaseUrl ?? null, apiProviderMode: apiProviderMode ?? null, apiProviderId: apiProviderId ?? null, apiProviderName: apiProviderName ?? null }));
  }
  async isOAuthPortInUse() { return this.invoke<boolean>('is_codex_oauth_port_in_use'); }
  async closeOAuthPort() { return this.invoke<number>('close_codex_oauth_port'); }
  async getConfigTomlPath() { return this.invoke<string>('get_codex_config_toml_path'); }
  async openConfigToml() { await this.invoke('open_codex_config_toml'); }
  async getQuickConfig() { return normalizeQuickConfig(await this.invoke('get_codex_quick_config')); }
  async saveQuickConfig(modelContextWindow?: number, autoCompactTokenLimit?: number) { return normalizeQuickConfig(await this.invoke('save_codex_quick_config', { modelContextWindow: modelContextWindow ?? null, autoCompactTokenLimit: autoCompactTokenLimit ?? null })); }

  async getLocalAccessState() { return normalizeCodexLocalAccessState(await this.invoke('codex_local_access_get_state')); }
  async saveLocalAccessAccounts(accountIds: string[], restrictFreeAccounts: boolean) { return normalizeCodexLocalAccessState(await this.invoke('codex_local_access_save_accounts', { accountIds, restrictFreeAccounts })); }
  async removeLocalAccessAccount(accountId: string) { return normalizeCodexLocalAccessState(await this.invoke('codex_local_access_remove_account', { accountId })); }
  async rotateLocalAccessApiKey() { return normalizeCodexLocalAccessState(await this.invoke('codex_local_access_rotate_api_key')); }
  async clearLocalAccessStats() { return normalizeCodexLocalAccessState(await this.invoke('codex_local_access_clear_stats')); }
  async prepareLocalAccessForRestart() { return normalizeCodexLocalAccessState(await this.invoke('codex_local_access_prepare_restart')); }
  async killLocalAccessPort() {
    const result = await this.invoke('codex_local_access_kill_port');
    const record = isRecord(result) ? result : {};
    return {
      killedCount: asNumber(readKey(record, 'killedCount', 'killed_count'), 0),
      state: normalizeCodexLocalAccessState(readKey(record, 'state')),
    };
  }
  async updateLocalAccessPort(port: number) { return normalizeCodexLocalAccessState(await this.invoke('codex_local_access_update_port', { port })); }
  async updateLocalAccessRoutingStrategy(strategy: CodexLocalAccessRoutingStrategy) { return normalizeCodexLocalAccessState(await this.invoke('codex_local_access_update_routing_strategy', { strategy })); }
  async setLocalAccessEnabled(enabled: boolean) { return normalizeCodexLocalAccessState(await this.invoke('codex_local_access_set_enabled', { enabled })); }
  async activateLocalAccess() { return normalizeCodexLocalAccessState(await this.invoke('codex_local_access_activate')); }

  async getInstanceDefaults() { return this.invoke<{ rootDir: string; defaultUserDataDir: string }>('codex_get_instance_defaults'); }
  async listInstances() { return parseJsonArray(await this.invoke('codex_list_instances')).map(normalizeCodexInstance); }
  async createInstance(payload: { name: string; userDataDir: string; workingDir?: string | null; extraArgs?: string; bindAccountId?: string | null; launchMode?: InstanceLaunchMode; copySourceInstanceId: string; initMode?: InstanceInitMode }) { return normalizeCodexInstance(await this.invoke('codex_create_instance', payload as unknown as Record<string, unknown>)); }
  async updateInstance(payload: { instanceId: string; name?: string; workingDir?: string | null; extraArgs?: string; bindAccountId?: string | null; followLocalAccount?: boolean; launchMode?: InstanceLaunchMode }) { return normalizeCodexInstance(await this.invoke('codex_update_instance', payload as unknown as Record<string, unknown>)); }
  async deleteInstance(instanceId: string) { await this.invoke('codex_delete_instance', { instanceId }); }
  async startInstance(instanceId: string) { return normalizeCodexInstance(await this.invoke('codex_start_instance', { instanceId })); }
  async stopInstance(instanceId: string) { return normalizeCodexInstance(await this.invoke('codex_stop_instance', { instanceId })); }
  async closeAllInstances() { await this.invoke('codex_close_all_instances'); }
  async openInstanceWindow(instanceId: string) { await this.invoke('codex_open_instance_window', { instanceId }); }
  async getInstanceQuickConfig(instanceId: string) { return normalizeQuickConfig(await this.invoke('codex_get_instance_quick_config', { instanceId })); }
  async saveInstanceQuickConfig(instanceId: string, modelContextWindow?: number, autoCompactTokenLimit?: number) { return normalizeQuickConfig(await this.invoke('codex_save_instance_quick_config', { instanceId, modelContextWindow: modelContextWindow ?? null, autoCompactTokenLimit: autoCompactTokenLimit ?? null })); }
  async openInstanceConfigToml(instanceId: string) { await this.invoke('codex_open_instance_config_toml', { instanceId }); }
  async getInstanceLaunchCommand(instanceId: string) { return this.invoke<{ instanceId: string; userDataDir: string; launchCommand: string }>('codex_get_instance_launch_command', { instanceId }); }
  async executeInstanceLaunchCommand(instanceId: string, terminal?: string) { return this.invoke<string>('codex_execute_instance_launch_command', { instanceId, terminal: terminal ?? null }); }
  async syncThreadsAcrossInstances() { return this.invoke<{ message: string }>('codex_sync_threads_across_instances'); }
  async repairSessionVisibilityAcrossInstances() { return this.invoke<{ message: string }>('codex_repair_session_visibility_across_instances'); }
  async listSessionsAcrossInstances() { return this.invoke<any[]>('codex_list_sessions_across_instances'); }
  async getSessionTokenStatsAcrossInstances(sessionIds: string[]) { return this.invoke<any[]>('codex_get_session_token_stats_across_instances', { sessionIds }); }
  async moveSessionsToTrashAcrossInstances(sessionIds: string[]) { return this.invoke<{ message: string }>('codex_move_sessions_to_trash_across_instances', { sessionIds }); }
  async listTrashedSessionsAcrossInstances() { return this.invoke<any[]>('codex_list_trashed_sessions_across_instances'); }
  async restoreSessionsFromTrashAcrossInstances(sessionIds: string[]) { return this.invoke<{ message: string }>('codex_restore_sessions_from_trash_across_instances', { sessionIds }); }

  async getWakeupCliStatus() { return this.invoke<CodexCliStatus>('codex_wakeup_get_cli_status'); }
  async updateWakeupRuntimeConfig(codexCliPath?: string, nodePath?: string) { return this.invoke<CodexCliStatus>('codex_wakeup_update_runtime_config', { codexCliPath: codexCliPath ?? null, nodePath: nodePath ?? null }); }
  async getWakeupOverview() { return this.invoke<CodexWakeupOverview>('codex_wakeup_get_overview'); }
  async saveWakeupState(enabled: boolean, tasks: CodexWakeupTask[], modelPresets: CodexWakeupModelPreset[], modelPresetMigrations?: string[]) { return this.invoke<CodexWakeupState>('codex_wakeup_save_state', { enabled, tasks, modelPresets, modelPresetMigrations: modelPresetMigrations ?? [] }); }
  async loadWakeupHistory() { return this.invoke<CodexWakeupHistoryItem[]>('codex_wakeup_load_history'); }
  async clearWakeupHistory() { await this.invoke('codex_wakeup_clear_history'); }
  async testWakeup(accountIds: string[], runId: string, prompt?: string, model?: string, modelDisplayName?: string, modelReasoningEffort?: CodexWakeupReasoningEffort, cancelScopeId?: string) {
    return this.invoke<CodexWakeupBatchResult>('codex_wakeup_test', { accountIds, runId, prompt: prompt ?? null, model: model ?? null, modelDisplayName: modelDisplayName ?? null, modelReasoningEffort: modelReasoningEffort ?? null, cancelScopeId: cancelScopeId ?? null });
  }
  async runWakeupTask(taskId: string, runId: string) { return this.invoke<CodexWakeupBatchResult>('codex_wakeup_run_task', { taskId, runId }); }
  async cancelWakeupScope(cancelScopeId: string) { await this.invoke('codex_wakeup_cancel_scope', { cancelScopeId }); }
  async releaseWakeupScope(cancelScopeId: string) { await this.invoke('codex_wakeup_release_scope', { cancelScopeId }); }

  async getGeneralConfig() { return this.invoke<Record<string, unknown>>('get_general_config'); }
  async saveGeneralConfig(config: Record<string, unknown>) { await this.invoke('save_general_config', config); }

  private async loadGroups() { return parseJsonArray(await this.invoke('load_codex_account_groups')).map(normalizeCodexGroup).sort((a, b) => (a.sortOrder ?? 0) - (b.sortOrder ?? 0)); }
  private async saveGroups(groups: ReturnType<typeof normalizeCodexGroup>[]) { await this.invoke('save_codex_account_groups', { data: JSON.stringify(groups, null, 2) }); }
  async listCodexAccountGroups() { return this.loadGroups(); }
  async createCodexGroup(name: string) {
    const groups = await this.loadGroups();
    const group = { id: `cgrp_${Date.now()}`, name: name.trim(), accountIds: [], sortOrder: groups.length ? Math.max(...groups.map((item) => item.sortOrder ?? 0)) + 1 : 1 };
    await this.saveGroups([...groups, group]);
    return group;
  }
  async renameCodexGroup(id: string, name: string) {
    const groups = await this.loadGroups();
    await this.saveGroups(groups.map((group) => group.id === id ? { ...group, name: name.trim() } : group));
  }
  async deleteCodexGroup(id: string) { await this.saveGroups((await this.loadGroups()).filter((group) => group.id !== id)); }
  async assignAccountsToCodexGroup(groupId: string, accountIds: string[]) {
    const groups = await this.loadGroups();
    const accountSet = new Set(accountIds);
    await this.saveGroups(groups.map((group) => {
      if (group.id === groupId) return { ...group, accountIds: Array.from(new Set([...group.accountIds, ...accountIds])) };
      return { ...group, accountIds: group.accountIds.filter((id) => !accountSet.has(id)) };
    }));
  }
  async removeAccountsFromCodexGroup(groupId: string, accountIds: string[]) {
    const removeSet = new Set(accountIds);
    await this.saveGroups((await this.loadGroups()).map((group) => group.id === groupId ? { ...group, accountIds: group.accountIds.filter((id) => !removeSet.has(id)) } : group));
  }

  private async loadProviders() { return parseJsonArray(await this.invoke('load_codex_model_providers')).map(normalizeCodexModelProvider); }
  private async saveProviders(providers: ReturnType<typeof normalizeCodexModelProvider>[]) { await this.invoke('save_codex_model_providers', { data: JSON.stringify(providers, null, 2) }); }
  async listCodexModelProviders() { return this.loadProviders(); }
  async createCodexModelProvider(payload: { name: string; baseUrl: string; website?: string; apiKeyUrl?: string; initialApiKey?: string; initialApiKeyName?: string }) {
    const providers = await this.loadProviders();
    const normalized = normalizeBaseUrl(payload.baseUrl);
    if (providers.some((provider) => normalizeBaseUrl(provider.baseUrl) === normalized)) throw new Error('PROVIDER_BASE_URL_EXISTS');
    const provider = {
      id: `cmp_${Date.now()}`,
      name: payload.name.trim(),
      baseUrl: payload.baseUrl.trim(),
      website: payload.website?.trim() || undefined,
      apiKeyUrl: payload.apiKeyUrl?.trim() || undefined,
      apiKeys: payload.initialApiKey ? [{ id: `cmk_${Date.now()}`, name: payload.initialApiKeyName?.trim() || undefined, apiKey: payload.initialApiKey.trim() }] : [],
    };
    await this.saveProviders([...providers, provider]);
    return provider;
  }
  async updateCodexModelProvider(id: string, payload: { name?: string; baseUrl?: string; website?: string; apiKeyUrl?: string }) {
    const providers = await this.loadProviders();
    const nextBaseUrl = payload.baseUrl?.trim();
    if (nextBaseUrl) {
      const normalized = normalizeBaseUrl(nextBaseUrl);
      if (providers.some((provider) => provider.id !== id && normalizeBaseUrl(provider.baseUrl) === normalized)) throw new Error('PROVIDER_BASE_URL_EXISTS');
    }
    await this.saveProviders(providers.map((provider) => provider.id === id ? { ...provider, ...payload, baseUrl: nextBaseUrl ?? provider.baseUrl } : provider));
  }
  async deleteCodexModelProvider(id: string) { await this.saveProviders((await this.loadProviders()).filter((provider) => provider.id !== id)); }
  async addApiKeyToCodexModelProvider(providerId: string, apiKey: string, name?: string) {
    const providers = await this.loadProviders();
    await this.saveProviders(providers.map((provider) => {
      if (provider.id !== providerId) return provider;
      if (provider.apiKeys.some((item) => item.apiKey.trim() === apiKey.trim())) return provider;
      return { ...provider, apiKeys: [...provider.apiKeys, { id: `cmk_${Date.now()}`, name: name?.trim() || undefined, apiKey: apiKey.trim() }] };
    }));
  }
  async removeApiKeyFromCodexModelProvider(providerId: string, apiKeyId: string) {
    await this.saveProviders((await this.loadProviders()).map((provider) => provider.id === providerId ? { ...provider, apiKeys: provider.apiKeys.filter((item) => item.id !== apiKeyId) } : provider));
  }
}

class MockBackendAdapter implements CodexBackendAdapter {
  private accounts: CodexAccount[] = [];
  private currentAccountId: string | null = null;
  private groups: ReturnType<typeof normalizeCodexGroup>[] = [];
  private providers: ReturnType<typeof normalizeCodexModelProvider>[] = [];
  private instances: InstanceProfile[] = [];
  private localAccess = normalizeCodexLocalAccessState({ enabled: false, running: false, port: 1455, base_url: 'http://localhost:1455', account_count: 0, local_api_key: 'sk-local-dev-key', stats: null });
  private generalConfig: Record<string, unknown> = {};
  private wakeupState: CodexWakeupState = { enabled: false, tasks: [], model_presets: [], model_preset_migrations: [] };

  async listAccounts() { return [...this.accounts]; }
  async getCurrentAccount() { return this.accounts.find((account) => account.id === this.currentAccountId) || null; }
  async switchAccount(accountId: string) {
    const account = this.accounts.find((item) => item.id === accountId);
    if (!account) throw new Error(`Account not found: ${accountId}`);
    this.currentAccountId = accountId;
    account.last_used = Math.floor(Date.now() / 1000);
    return account;
  }
  async deleteAccount(accountId: string) { await this.deleteAccounts([accountId]); }
  async deleteAccounts(accountIds: string[]) {
    this.accounts = this.accounts.filter((account) => !accountIds.includes(account.id));
    this.groups = this.groups.map((group) => ({ ...group, accountIds: group.accountIds.filter((id) => !accountIds.includes(id)) }));
    if (this.currentAccountId && accountIds.includes(this.currentAccountId)) this.currentAccountId = null;
  }
  async importFromLocal(): Promise<CodexAccount> { throw new Error('Local auth import requires a native backend adapter'); }
  async importFromJson(jsonContent: string) {
    const parsed = JSON.parse(jsonContent);
    const imported = normalizeCodexAccounts(Array.isArray(parsed) ? parsed : [parsed]);
    this.accounts.push(...imported);
    if (!this.currentAccountId && imported[0]) this.currentAccountId = imported[0].id;
    return imported;
  }
  async importFromFiles(_filePaths: string[]): Promise<CodexFileImportResult> { throw new Error('File import requires a native backend adapter'); }
  async exportAccounts(accountIds: string[]) { return JSON.stringify(this.accounts.filter((account) => accountIds.includes(account.id)), null, 2); }
  async refreshQuota(accountId: string) {
    const quota = { hourly_percentage: 50, weekly_percentage: 60 };
    this.accounts = this.accounts.map((account) => account.id === accountId ? { ...account, quota } : account);
    return quota;
  }
  async refreshAllQuotas() {
    for (const account of this.accounts.filter((item) => !item.auth_mode || item.auth_mode === 'oauth')) await this.refreshQuota(account.id);
    return this.accounts.length;
  }
  async refreshAccountProfile(accountId: string) {
    const account = this.accounts.find((item) => item.id === accountId);
    if (!account) throw new Error(`Account not found: ${accountId}`);
    return account;
  }
  async updateAccountName(accountId: string, name: string) {
    this.accounts = this.accounts.map((account) => account.id === accountId ? { ...account, email: name.trim() || account.email, account_name: name.trim() || account.account_name } : account);
    return this.refreshAccountProfile(accountId);
  }
  async updateAccountTags(accountId: string, tags: string[]) {
    this.accounts = this.accounts.map((account) => account.id === accountId ? { ...account, tags } : account);
    return this.refreshAccountProfile(accountId);
  }
  async updateApiKeyCredentials(accountId: string, apiKey: string, apiBaseUrl?: string, apiProviderMode?: CodexApiProviderMode, apiProviderId?: string, apiProviderName?: string) {
    this.accounts = this.accounts.map((account) => account.id === accountId ? { ...account, openai_api_key: apiKey, api_base_url: apiBaseUrl, api_provider_mode: apiProviderMode, api_provider_id: apiProviderId, api_provider_name: apiProviderName } : account);
    return this.refreshAccountProfile(accountId);
  }
  async startOAuthLogin() { return { loginId: `login_${Date.now()}`, authUrl: 'https://auth.openai.com/authorize?mock=true' }; }
  async completeOAuthLogin(_loginId: string) {
    const account = normalizeCodexAccount({ id: `codex_${Date.now()}`, email: `oauth_user_${Date.now()}@example.com`, auth_mode: 'oauth', tokens: { id_token: 'mock_id_token', access_token: 'mock_access_token', refresh_token: 'mock_refresh_token' }, plan_type: 'PLUS', created_at: Date.now(), last_used: Date.now() });
    this.accounts.push(account);
    this.currentAccountId = account.id;
    return account;
  }
  async cancelOAuthLogin(_loginId?: string) {}
  async submitOAuthCallbackUrl(_loginId: string, _callbackUrl: string) {}
  async addAccountWithToken(idToken: string, accessToken: string, refreshToken?: string) {
    const account = normalizeCodexAccount({ id: `codex_${Date.now()}`, email: 'token_imported@example.com', auth_mode: 'oauth', tokens: { id_token: idToken, access_token: accessToken, refresh_token: refreshToken }, created_at: Date.now(), last_used: Date.now() });
    this.accounts.push(account);
    return account;
  }
  async addAccountWithApiKey(apiKey: string, apiBaseUrl?: string, apiProviderMode?: CodexApiProviderMode, apiProviderId?: string, apiProviderName?: string) {
    const account = normalizeCodexAccount({ id: `codex_${Date.now()}`, display_name: apiProviderName || 'OpenAI API Key', auth_mode: 'apikey', tokens: {}, api_key: apiKey, base_url: apiBaseUrl, api_provider_mode: apiProviderMode || 'openai', provider_id: apiProviderId, provider_name: apiProviderName, created_at: Date.now(), last_used: Date.now() });
    this.accounts.push(account);
    return account;
  }
  async isOAuthPortInUse() { return false; }
  async closeOAuthPort() { return 0; }
  async getConfigTomlPath() { return '~/.codex/config.toml'; }
  async openConfigToml() {}
  async getQuickConfig() { return normalizeQuickConfig({ model_context_window: 1_000_000, model_auto_compact_token_limit: 900_000 }); }
  async saveQuickConfig(modelContextWindow?: number, autoCompactTokenLimit?: number) { return normalizeQuickConfig({ model_context_window: modelContextWindow, model_auto_compact_token_limit: autoCompactTokenLimit }); }
  async getLocalAccessState() { return this.localAccess; }
  async saveLocalAccessAccounts(accountIds: string[], restrictFreeAccounts: boolean) {
    this.localAccess = { ...this.localAccess, collection: { ...(this.localAccess.collection ?? normalizeLocalAccessCollection(null, { port: 1455, local_api_key: 'sk-local-dev-key' })!), accountIds, restrictFreeAccounts }, memberCount: accountIds.length };
    return this.localAccess;
  }
  async removeLocalAccessAccount(accountId: string) {
    const ids = (this.localAccess.collection?.accountIds ?? []).filter((id) => id !== accountId);
    return this.saveLocalAccessAccounts(ids, this.localAccess.collection?.restrictFreeAccounts ?? true);
  }
  async rotateLocalAccessApiKey() {
    this.localAccess = { ...this.localAccess, collection: { ...(this.localAccess.collection ?? normalizeLocalAccessCollection(null, { port: 1455 })!), apiKey: `sk-local-${Date.now()}` } };
    return this.localAccess;
  }
  async clearLocalAccessStats() {
    this.localAccess = normalizeCodexLocalAccessState({ ...this.localAccess, stats: null });
    return this.localAccess;
  }
  async prepareLocalAccessForRestart() { return this.localAccess; }
  async killLocalAccessPort() { return { killedCount: 0, state: this.localAccess }; }
  async updateLocalAccessPort(port: number) {
    this.localAccess = { ...this.localAccess, baseUrl: `http://localhost:${port}`, collection: { ...(this.localAccess.collection ?? normalizeLocalAccessCollection(null, { local_api_key: 'sk-local-dev-key' })!), port } };
    return this.localAccess;
  }
  async updateLocalAccessRoutingStrategy(strategy: CodexLocalAccessRoutingStrategy) {
    this.localAccess = { ...this.localAccess, collection: { ...(this.localAccess.collection ?? normalizeLocalAccessCollection(null, { port: 1455, local_api_key: 'sk-local-dev-key' })!), routingStrategy: strategy } };
    return this.localAccess;
  }
  async setLocalAccessEnabled(enabled: boolean) {
    this.localAccess = { ...this.localAccess, running: enabled, collection: { ...(this.localAccess.collection ?? normalizeLocalAccessCollection(null, { port: 1455, local_api_key: 'sk-local-dev-key' })!), enabled } };
    return this.localAccess;
  }
  async activateLocalAccess() { return this.setLocalAccessEnabled(true); }
  async getInstanceDefaults() { return { rootDir: '~/.codex/instances', defaultUserDataDir: '~/.codex' }; }
  async listInstances() { return this.instances; }
  async createInstance(payload: { name: string; userDataDir: string; workingDir?: string | null; extraArgs?: string; bindAccountId?: string | null; launchMode?: InstanceLaunchMode; copySourceInstanceId: string; initMode?: InstanceInitMode }) {
    const instance = normalizeCodexInstance({ id: `inst_${Date.now()}`, name: payload.name, userDataDir: payload.userDataDir, workingDir: payload.workingDir, extraArgs: payload.extraArgs, bindAccountId: payload.bindAccountId, launchMode: payload.launchMode, createdAt: Date.now(), running: false });
    this.instances.push(instance);
    return instance;
  }
  async updateInstance(payload: { instanceId: string; name?: string; workingDir?: string | null; extraArgs?: string; bindAccountId?: string | null; followLocalAccount?: boolean; launchMode?: InstanceLaunchMode }) {
    this.instances = this.instances.map((instance) => instance.id === payload.instanceId ? { ...instance, ...payload } : instance);
    return this.instances.find((instance) => instance.id === payload.instanceId)!;
  }
  async deleteInstance(instanceId: string) { this.instances = this.instances.filter((instance) => instance.id !== instanceId); }
  async startInstance(instanceId: string) {
    this.instances = this.instances.map((instance) => instance.id === instanceId ? { ...instance, running: true } : instance);
    return this.instances.find((instance) => instance.id === instanceId)!;
  }
  async stopInstance(instanceId: string) {
    this.instances = this.instances.map((instance) => instance.id === instanceId ? { ...instance, running: false } : instance);
    return this.instances.find((instance) => instance.id === instanceId)!;
  }
  async closeAllInstances() { this.instances = this.instances.map((instance) => ({ ...instance, running: false })); }
  async openInstanceWindow(_instanceId: string) {}
  async getInstanceQuickConfig(_instanceId: string) { return this.getQuickConfig(); }
  async saveInstanceQuickConfig(_instanceId: string, modelContextWindow?: number, autoCompactTokenLimit?: number) { return this.saveQuickConfig(modelContextWindow, autoCompactTokenLimit); }
  async openInstanceConfigToml(_instanceId: string) {}
  async getInstanceLaunchCommand(instanceId: string) { return { instanceId, userDataDir: this.instances.find((item) => item.id === instanceId)?.userDataDir ?? '', launchCommand: 'codex' }; }
  async executeInstanceLaunchCommand(_instanceId: string, _terminal?: string) { return 'Command prepared'; }
  async syncThreadsAcrossInstances() { return { message: 'Threads synced' }; }
  async repairSessionVisibilityAcrossInstances() { return { message: 'Visibility repaired' }; }
  async listSessionsAcrossInstances() { return []; }
  async getSessionTokenStatsAcrossInstances(_sessionIds: string[]) { return []; }
  async moveSessionsToTrashAcrossInstances(_sessionIds: string[]) { return { message: 'Moved to trash' }; }
  async listTrashedSessionsAcrossInstances() { return []; }
  async restoreSessionsFromTrashAcrossInstances(_sessionIds: string[]) { return { message: 'Restored' }; }
  async getWakeupCliStatus(): Promise<CodexCliStatus> { return { available: true, required_runtime_paths: [], checked_at: Math.floor(Date.now() / 1000), install_hints: [] }; }
  async updateWakeupRuntimeConfig(codexCliPath?: string, nodePath?: string): Promise<CodexCliStatus> { return { available: true, configured_codex_cli_path: codexCliPath, configured_node_path: nodePath, required_runtime_paths: [], checked_at: Math.floor(Date.now() / 1000), install_hints: [] }; }
  async getWakeupOverview(): Promise<CodexWakeupOverview> { return { runtime: await this.getWakeupCliStatus(), state: this.wakeupState, history: [] }; }
  async saveWakeupState(enabled: boolean, tasks: CodexWakeupTask[], modelPresets: CodexWakeupModelPreset[], modelPresetMigrations?: string[]) { this.wakeupState = { enabled, tasks, model_presets: modelPresets, model_preset_migrations: modelPresetMigrations || [] }; return this.wakeupState; }
  async loadWakeupHistory() { return []; }
  async clearWakeupHistory() {}
  async testWakeup(_accountIds: string[], runId: string): Promise<CodexWakeupBatchResult> { return { run_id: runId, runtime: await this.getWakeupCliStatus(), records: [], success_count: 0, failure_count: 0 }; }
  async runWakeupTask(_taskId: string, runId: string) { return this.testWakeup([], runId); }
  async cancelWakeupScope(_cancelScopeId: string) {}
  async releaseWakeupScope(_cancelScopeId: string) {}
  async getGeneralConfig() { return this.generalConfig; }
  async saveGeneralConfig(config: Record<string, unknown>) { this.generalConfig = { ...this.generalConfig, ...config }; }
  async listCodexAccountGroups() { return [...this.groups]; }
  async createCodexGroup(name: string) { const group = { id: `cgrp_${Date.now()}`, name: name.trim(), accountIds: [], sortOrder: this.groups.length + 1 }; this.groups.push(group); return group; }
  async renameCodexGroup(id: string, name: string) { this.groups = this.groups.map((group) => group.id === id ? { ...group, name: name.trim() } : group); }
  async deleteCodexGroup(id: string) { this.groups = this.groups.filter((group) => group.id !== id); }
  async assignAccountsToCodexGroup(groupId: string, accountIds: string[]) { this.groups = this.groups.map((group) => group.id === groupId ? { ...group, accountIds: Array.from(new Set([...group.accountIds, ...accountIds])) } : { ...group, accountIds: group.accountIds.filter((id) => !accountIds.includes(id)) }); }
  async removeAccountsFromCodexGroup(groupId: string, accountIds: string[]) { this.groups = this.groups.map((group) => group.id === groupId ? { ...group, accountIds: group.accountIds.filter((id) => !accountIds.includes(id)) } : group); }
  async listCodexModelProviders() { return [...this.providers]; }
 async createCodexModelProvider(payload: { name: string; baseUrl: string; website?: string; apiKeyUrl?: string; initialApiKey?: string; initialApiKeyName?: string }) {
    const provider = normalizeCodexModelProvider({ id: `cmp_${Date.now()}`, name: payload.name, baseUrl: payload.baseUrl, website: payload.website, apiKeyUrl: payload.apiKeyUrl, apiKeys: payload.initialApiKey ? [{ id: `cmk_${Date.now()}`, name: payload.initialApiKeyName, apiKey: payload.initialApiKey }] : [] });
    this.providers.push(provider);
    return provider;
  }
  async updateCodexModelProvider(id: string, payload: { name?: string; baseUrl?: string; website?: string; apiKeyUrl?: string }) { this.providers = this.providers.map((provider) => provider.id === id ? { ...provider, ...payload } : provider); }
  async deleteCodexModelProvider(id: string) { this.providers = this.providers.filter((provider) => provider.id !== id); }
  async addApiKeyToCodexModelProvider(providerId: string, apiKey: string, name?: string) { this.providers = this.providers.map((provider) => provider.id === providerId ? { ...provider, apiKeys: [...provider.apiKeys, { id: `cmk_${Date.now()}`, name, apiKey }] } : provider); }
  async removeApiKeyFromCodexModelProvider(providerId: string, apiKeyId: string) { this.providers = this.providers.map((provider) => provider.id === providerId ? { ...provider, apiKeys: provider.apiKeys.filter((item) => item.id !== apiKeyId) } : provider); }
}

function getTauriInvoke(): InvokeFn | null {
  if (typeof window === 'undefined') return null;
  return window.__TAURI__?.core?.invoke ?? window.__TAURI_INTERNALS__?.invoke ?? null;
}

export function createRuntimeBackendAdapter(): CodexBackendAdapter {
  if (typeof window !== 'undefined' && window.__OAUTHCODEX_BACKEND__) return window.__OAUTHCODEX_BACKEND__;
  const invoke = getTauriInvoke();
  if (invoke) return new TauriBackendAdapter(invoke);
  const env = (import.meta as unknown as { env?: Record<string, unknown> }).env;
  if (env?.DEV || env?.MODE === 'test') return new MockBackendAdapter();
  return new UnavailableBackendAdapter();
}

let currentAdapter: CodexBackendAdapter = createRuntimeBackendAdapter();

export function setAdapter(adapter: CodexBackendAdapter) {
  currentAdapter = adapter;
}

export function resetAdapter() {
  currentAdapter = createRuntimeBackendAdapter();
}

export function getAdapter(): CodexBackendAdapter {
  return currentAdapter;
}

export const codexClient = new Proxy({} as CodexBackendAdapter, {
  get(_target, prop: string) {
    const value = (currentAdapter as unknown as Record<string, unknown>)[prop];
    if (typeof value !== 'function') {
      throw new Error(`Codex backend method is not implemented: ${prop}`);
    }
    return (...args: unknown[]) => value.apply(currentAdapter, args);
  },
});
