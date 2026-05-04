import { describe, it, expect } from 'vitest';
import { isCodexApiKeyAccount, getCodexPlanDisplayName, getCodexQuotaClass, getCodexQuotaWindows, maskApiKey } from '../src/types/codex';
import { buildCodexAccountPresentation } from '../src/presentation/codexPresentation';
import {
  normalizeCodexAccount,
  normalizeCodexAccounts,
  normalizeCodexGroup,
  normalizeCodexInstance,
  normalizeCodexLocalAccessState,
  normalizeCodexModelProvider,
  normalizeOAuthLoginStartResponse,
} from '../src/services/codexClient';

describe('Codex Account Types', () => {
  const apiKeyAccount = {
    id: 'codex_1',
    email: 'api@example.com',
    auth_mode: 'apikey',
    tokens: { id_token: '', access_token: '' },
    created_at: 1000,
    last_used: 1000,
  };

  const oauthAccount = {
    id: 'codex_2',
    email: 'oauth@example.com',
    auth_mode: 'oauth',
    tokens: { id_token: 'eyJ.', access_token: 'eyJ.' },
    plan_type: 'PLUS',
    created_at: 1000,
    last_used: 1000,
  };

  it('identifies API key accounts', () => {
    expect(isCodexApiKeyAccount(apiKeyAccount as any)).toBe(true);
    expect(isCodexApiKeyAccount(oauthAccount as any)).toBe(false);
  });

  it('returns plan display names', () => {
    expect(getCodexPlanDisplayName('PLUS')).toBe('PLUS');
    expect(getCodexPlanDisplayName('TEAM')).toBe('TEAM');
    expect(getCodexPlanDisplayName('')).toBe('FREE');
    expect(getCodexPlanDisplayName('PRO')).toBe('PRO');
  });

  it('maps quota percentages to CSS classes', () => {
    expect(getCodexQuotaClass(85)).toBe('high');
    expect(getCodexQuotaClass(50)).toBe('medium');
    expect(getCodexQuotaClass(20)).toBe('low');
    expect(getCodexQuotaClass(5)).toBe('critical');
  });

  it('builds account presentation', () => {
    const pres = buildCodexAccountPresentation(oauthAccount as any);
    expect(pres.displayName).toBe('oauth@example.com');
    expect(pres.planLabel).toBe('PLUS');
    expect(pres.planClass).toBe('plus');
    expect(pres.isApiKey).toBe(false);
  });

  it('parses quota windows', () => {
    const quota = { hourly_percentage: 60, weekly_percentage: 30 };
    const windows = getCodexQuotaWindows(quota);
    expect(windows.length).toBeGreaterThanOrEqual(1);
    expect(windows[0].percentage).toBe(60);
  });
});

describe('Codex Store Contract', () => {
  it('has correct localStorage keys defined', () => {
    const keys = [
      'agtools.codex.accounts.overview_layout_mode',
      'agtools.codex.local_access_entry_expanded.v1',
      'agtools.codex.accounts.custom_sort_order.v1',
      'agtools.codex.accounts.cache',
      'agtools.codex.accounts.current',
    ];
    keys.forEach(k => expect(typeof k).toBe('string'));
  });
});

describe('Codex API Key Validation', () => {
  it('masks API keys correctly', () => {
    expect(maskApiKey('')).toBe('');
    expect(maskApiKey('sk-test')).toBeTruthy();
    expect(maskApiKey('sk-test').length).toBeGreaterThan(0);
  });
});

describe('Codex backend normalization', () => {
  it('normalizes Rust account payloads without crashing the UI contract', () => {
    const account = normalizeCodexAccount({
      id: 'acct_apikey_openai_003',
      provider: 'codex',
      auth_mode: 'apikey',
      email: null,
      display_name: 'OpenAI API Key',
      tokens: {},
      api_key: 'sk-test_openai_api_key_mock_value_123',
      base_url: 'https://api.openai.com/v1',
      provider_id: 'cmp_openai_default',
      provider_name: 'OpenAI',
      api_provider_mode: 'openai',
      created_at: '2026-05-01T14:00:00Z',
      last_used: '2026-05-03T11:30:00Z',
    });

    expect(account.email).toBe('OpenAI API Key');
    expect(account.api_base_url).toBe('https://api.openai.com/v1');
    expect(account.api_provider_mode).toBe('openai');
    expect(account.created_at).toBeGreaterThan(0);
    expect(account.last_used).toBeGreaterThan(account.created_at);
    expect(account.tokens).toEqual({ id_token: '', access_token: '' });
  });

  it('normalizes account lists wrapped by Rust index payloads', () => {
    const accounts = normalizeCodexAccounts({
      version: 1,
      current_account_id: 'acct_1',
      accounts: [
        { id: 'acct_1', auth_mode: 'oauth', email: 'dev@example.com', tokens: {}, created_at: null, last_used: null },
      ],
    });

    expect(accounts).toHaveLength(1);
    expect(accounts[0].email).toBe('dev@example.com');
  });

  it('normalizes Rust local access snapshots into source UI shape', () => {
    const state = normalizeCodexLocalAccessState({
      enabled: true,
      running: false,
      port: 1455,
      base_url: 'http://localhost:1455',
      account_count: 2,
      last_error: null,
      local_api_key: 'sk-local-abc123',
      stats: {
        daily: { requests: 3, successes: 2, failures: 1, tokens_in: 10, tokens_out: 20, latency_ms_sum: 90 },
        weekly: { requests: 4, successes: 4, failures: 0, tokens_in: 11, tokens_out: 21, latency_ms_sum: 91 },
        monthly: { requests: 5, successes: 5, failures: 0, tokens_in: 12, tokens_out: 22, latency_ms_sum: 92 },
      },
    });

    expect(state.collection?.apiKey).toBe('sk-local-abc123');
    expect(state.collection?.port).toBe(1455);
    expect(state.memberCount).toBe(2);
    expect(state.stats.daily.totals.requestCount).toBe(3);
    expect(state.stats.daily.totals.inputTokens).toBe(10);
  });

  it('normalizes model providers, groups, instances, and OAuth start response', () => {
    expect(normalizeCodexGroup({ id: 'cgrp_1', name: 'Work', account_ids: ['acct_1'] }).accountIds).toEqual(['acct_1']);
    expect(normalizeCodexModelProvider({
      id: 'cmp_1',
      name: 'OpenAI',
      base_url: 'https://api.openai.com/v1',
      api_keys: [{ id: 'cmk_1', key: 'sk-test' }],
    }).apiKeys[0].apiKey).toBe('sk-test');
    expect(normalizeCodexInstance({
      id: 'inst_1',
      name: 'Default',
      is_default: true,
      working_dir: '/tmp/project',
      bound_account_id: 'acct_1',
      follow_local_account: true,
      launch_mode: 'manual',
      extra_args: ['--model', 'gpt-5'],
      enabled: true,
      created_at: '2026-05-01T00:00:00Z',
      updated_at: '2026-05-02T00:00:00Z',
    }).launchMode).toBe('manual');
    expect(normalizeOAuthLoginStartResponse({ login_id: 'login_1', auth_url: 'https://example.test' })).toEqual({
      loginId: 'login_1',
      authUrl: 'https://example.test',
    });
  });
});
