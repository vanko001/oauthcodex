import type { CodexAccount, CodexQuotaWindow, CodexSubscriptionPresentation } from '../types/codex';

export interface CodexAccountPresentation {
  displayName: string;
  planLabel: string;
  planClass: string;
  planFilterKey: string;
  quotaWindows: CodexQuotaWindow[];
  subscription: CodexSubscriptionPresentation | null;
  isApiKey: boolean;
  isTeamPlan: boolean;
}

export function buildCodexAccountPresentation(account: CodexAccount): CodexAccountPresentation {
  const planType = account.plan_type || 'FREE';
  const planLabel = getPlanLabel(planType);
  const planClass = getPlanClass(planType);
  return {
    displayName: account.email || account.id,
    planLabel,
    planClass,
    planFilterKey: planLabel.toUpperCase(),
    quotaWindows: [],
    subscription: null,
    isApiKey: (account.auth_mode || '').toLowerCase() === 'apikey',
    isTeamPlan: ['TEAM', 'ENTERPRISE', 'BUSINESS', 'EDU'].includes(planLabel),
  };
}

function getPlanLabel(planType: string): string {
  const upper = planType.toUpperCase();
  if (upper.includes('TEAM')) return 'TEAM';
  if (upper.includes('ENTERPRISE')) return 'ENTERPRISE';
  if (upper.includes('PLUS')) return 'PLUS';
  if (upper.includes('PRO')) return 'PRO';
  if (upper.includes('EDU')) return 'EDU';
  if (upper.includes('FREE')) return 'FREE';
  return upper;
}

function getPlanClass(planType: string): string {
  const lower = planType.toLowerCase();
  if (lower.includes('plus')) return 'plus';
  if (lower.includes('pro')) return 'pro';
  if (lower.includes('team')) return 'team';
  if (lower.includes('enterprise')) return 'enterprise';
  if (lower.includes('edu')) return 'edu';
  return 'free';
}

export function maskSensitiveValue(value?: string | null, enabled?: boolean): string {
  if (!enabled || !value) return value || '';
  if (value.length <= 4) return '****';
  return `${value.slice(0, 2)}****${value.slice(-2)}`;
}

export const CODEX_CODE_REVIEW_QUOTA_VISIBILITY_CHANGED_EVENT = 'codex-code-review-quota-visibility-changed';
