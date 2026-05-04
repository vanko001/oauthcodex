export type CodexApiProviderMode = 'openai_builtin' | 'openai' | 'custom' | 'azure';

export type CodexOverviewLayoutMode = 'compact' | 'list' | 'grid';
export type CodexLaunchCredentialKind = 'api-key' | 'api-service' | 'account';
export type CodexLaunchCredentialType = 'api' | 'account';
export type CodexExportFormat = 'cockpit_tools' | 'sub2api' | 'cpa';

export interface CodexQuickConfig {
  context_window_1m: boolean;
  auto_compact_token_limit: number;
  detected_model_context_window?: number;
  detected_auto_compact_token_limit?: number;
}

export interface CodexAccount {
  id: string;
  provider?: string;
  email: string;
  display_name?: string;
  auth_mode?: string;
  api_key?: string;
  base_url?: string;
  openai_api_key?: string;
  api_base_url?: string;
  api_provider_mode?: CodexApiProviderMode;
  api_provider_id?: string;
  api_provider_name?: string;
  provider_id?: string;
  provider_name?: string;
  user_id?: string;
  plan_type?: string;
  subscription_active_until?: string;
  auth_file_plan_type?: string;
  account_id?: string;
  organization_id?: string;
  account_name?: string;
  account_structure?: string;
  tokens: CodexTokens;
  token_generation?: number;
  token_updated_at?: number;
  token_source_mode?: string;
  requires_reauth?: boolean;
  reauth_reason?: string;
  quota?: CodexQuota;
  quota_error?: CodexQuotaErrorInfo;
  tags?: string[];
  created_at: number;
  last_used: number;
}

export interface CodexQuotaErrorInfo {
  code?: string;
  message: string;
  timestamp: number;
}

export interface CodexTokens {
  id_token: string;
  access_token: string;
  refresh_token?: string;
}

export interface CodexQuota {
  hourly_percentage: number;
  hourly_reset_time?: number;
  hourly_window_minutes?: number;
  hourly_window_present?: boolean;
  weekly_percentage: number;
  weekly_reset_time?: number;
  weekly_window_minutes?: number;
  weekly_window_present?: boolean;
  code_review_quota?: unknown;
  raw_data?: unknown;
}

export interface CodexWorkspace {
  id: string;
  title: string;
  role?: string;
  is_default?: boolean;
}

export interface CodexAuthMetadata {
  chatgptAccountId?: string;
  authProvider?: string;
  userId?: string;
  workspaces: CodexWorkspace[];
}

export interface CodexCodeReviewQuotaMetric {
  percentage: number;
  label: string;
  resetTime?: number;
}

export interface CodexInstanceThreadSyncItem {
  instanceId: string;
  instanceName: string;
  addedThreadCount: number;
  backupDir?: string | null;
}

export interface CodexInstanceThreadSyncSummary {
  instanceCount: number;
  threadUniverseCount: number;
  mutatedInstanceCount: number;
  totalSyncedThreadCount: number;
  items: CodexInstanceThreadSyncItem[];
  backupDirs: string[];
  message: string;
}

export interface CodexSessionVisibilityRepairItem {
  instanceId: string;
  instanceName: string;
  targetProvider: string;
  changedRolloutFileCount: number;
  updatedSqliteRowCount: number;
  skippedSqliteFile: boolean;
  backupDir?: string | null;
  running: boolean;
}

export interface CodexSessionVisibilityRepairSummary {
  instanceCount: number;
  mutatedInstanceCount: number;
  changedRolloutFileCount: number;
  updatedSqliteRowCount: number;
  skippedSqliteFileCount: number;
  items: CodexSessionVisibilityRepairItem[];
  backupDirs: string[];
  message: string;
}

export interface CodexSessionLocation {
  instanceId: string;
  instanceName: string;
  running: boolean;
}

export interface CodexSessionRecord {
  sessionId: string;
  title: string;
  cwd: string;
  updatedAt?: number | null;
  locationCount: number;
  locations: CodexSessionLocation[];
}

export interface CodexSessionTokenStats {
  sessionId: string;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
}

export interface CodexSessionTrashSummary {
  requestedSessionCount: number;
  trashedSessionCount: number;
  trashedInstanceCount: number;
  trashDirs: string[];
  message: string;
}

export interface CodexTrashedSessionLocation {
  instanceId: string;
  instanceName: string;
}

export interface CodexTrashedSessionRecord {
  sessionId: string;
  title: string;
  cwd: string;
  deletedAt?: number | null;
  locationCount: number;
  locations: CodexTrashedSessionLocation[];
}

export interface CodexSessionRestoreSummary {
  requestedSessionCount: number;
  restoredSessionCount: number;
  restoredInstanceCount: number;
  message: string;
}

export interface CodexOAuthLoginStartResponse {
  loginId: string;
  authUrl: string;
}

export interface CodexFileImportResult {
  imported: CodexAccount[];
  failed: { email: string; error: string }[];
}

export interface CodexSubscriptionPresentation {
  bucket: 'missing' | 'expired' | 'within_24h' | 'within_7d' | 'within_30d' | 'active';
  tone: 'missing' | 'expired' | 'warning' | 'active';
  valueText: string;
  detailText: string;
  titleText: string;
  timestampMs: number | null;
}

export interface CodexQuotaWindow {
  id: 'primary' | 'secondary';
  label: string;
  percentage: number;
  resetTime?: number;
  windowMinutes?: number;
}

export interface CodexApiSwitchNoticeContext {
  from: CodexLaunchCredentialKind;
  to: CodexLaunchCredentialKind;
}

export function isCodexApiKeyAccount(account: CodexAccount): boolean {
  return (account.auth_mode || '').trim().toLowerCase() === 'apikey';
}

export function getCodexPlanDisplayName(planType?: string): string {
  if (!planType) return 'FREE';
  const upper = planType.toUpperCase();
  if (upper.includes('TEAM')) return 'TEAM';
  if (upper.includes('ENTERPRISE')) return 'ENTERPRISE';
  if (upper.includes('PLUS')) return 'PLUS';
  if (upper.includes('PRO')) return 'PRO';
  return upper;
}

export function isCodexExplicitFreePlanType(planType?: string): boolean {
  if (!planType) return false;
  return (planType || '').trim().toLowerCase() === 'free';
}

export function isCodexTeamLikePlan(planType?: string): boolean {
  if (!planType) return false;
  const upper = planType.toUpperCase();
  return upper.includes('TEAM') || upper.includes('BUSINESS') || upper.includes('ENTERPRISE') || upper.includes('EDU');
}

export function hasCodexAccountName(account: CodexAccount): boolean {
  return typeof account.account_name === 'string' && account.account_name.trim().length > 0;
}

export function hasCodexAccountStructure(account: CodexAccount): boolean {
  return typeof account.account_structure === 'string' && account.account_structure.trim().length > 0;
}

export function getAuthMetadata(account: CodexAccount): CodexAuthMetadata {
  return {
    chatgptAccountId: account.account_id,
    authProvider: undefined,
    userId: account.user_id,
    workspaces: [],
  };
}

export function getCodexPlanBadgeLabel(account: CodexAccount): string {
  return getCodexPlanDisplayName(account.plan_type);
}

export function getCodexPlanBadgeClass(account: CodexAccount): string {
  const key = (account.plan_type || '').trim().toLowerCase();
  if (key.includes('plus')) return 'plus';
  if (key.includes('pro')) return 'pro';
  if (key.includes('team')) return 'team';
  if (key.includes('enterprise')) return 'enterprise';
  if (key.includes('edu')) return 'edu';
  return 'free';
}

export function getCodexPlanFilterKey(account: CodexAccount): string {
  return getCodexPlanDisplayName(account.plan_type).toUpperCase();
}

export function getCodexQuotaClass(percentage: number): string {
  if (percentage >= 80) return 'high';
  if (percentage >= 40) return 'medium';
  if (percentage >= 10) return 'low';
  return 'critical';
}

export function getCodexQuotaWindowLabel(windowMinutes?: number, fallback: 'hourly' | 'weekly' = 'hourly'): string {
  if (!windowMinutes || windowMinutes <= 0) return fallback === 'weekly' ? 'Weekly' : '5h';
  if (windowMinutes >= 10080) return 'Weekly';
  if (windowMinutes >= 1440) return `${Math.ceil(windowMinutes / 1440)}d`;
  if (windowMinutes >= 60) return `${Math.ceil(windowMinutes / 60)}h`;
  return `${Math.ceil(windowMinutes)}m`;
}

export function getCodexQuotaWindows(quota?: CodexQuota): CodexQuotaWindow[] {
  if (!quota) return [];
  const windows: CodexQuotaWindow[] = [];
  if (quota.hourly_window_present !== false) {
    windows.push({
      id: 'primary',
      label: getCodexQuotaWindowLabel(quota.hourly_window_minutes, 'hourly'),
      percentage: quota.hourly_percentage,
      resetTime: quota.hourly_reset_time,
      windowMinutes: quota.hourly_window_minutes,
    });
  }
  if (quota.weekly_window_present !== false) {
    windows.push({
      id: 'secondary',
      label: getCodexQuotaWindowLabel(quota.weekly_window_minutes, 'weekly'),
      percentage: quota.weekly_percentage,
      resetTime: quota.weekly_reset_time,
      windowMinutes: quota.weekly_window_minutes,
    });
  }
  return windows;
}

export function formatResetTime(resetTime?: number, _t?: (k: string) => string): string {
  if (!resetTime) return '';
  const now = Math.floor(Date.now() / 1000);
  const diff = resetTime - now;
  if (diff <= 0) return 'Resetting...';
  const totalMinutes = Math.floor(diff / 60);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  let parts = [];
  if (hours > 0) parts.push(`${hours}h`);
  if (minutes > 0) parts.push(`${minutes}m`);
  return parts.join(' ') || '<1m';
}

export function formatResetTimeAbsolute(resetTime?: number): string {
  if (!resetTime) return '';
  const d = new Date(resetTime * 1000);
  return `${String(d.getMonth() + 1).padStart(2, '0')}/${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
}

export function getCodexCodeReviewQuotaMetric(_quota?: CodexQuota): CodexCodeReviewQuotaMetric | null {
  return null;
}

export function maskApiKey(value: string): string {
  const trimmed = value.trim();
  if (!trimmed) return '';
  if (trimmed.startsWith('sk-')) return 'sk-\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022';
  return '\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022';
}
