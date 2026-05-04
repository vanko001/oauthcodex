import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { CodexAccountsPage } from '../src/pages/CodexAccountsPage';
import { CodexSettingsPage } from '../src/pages/CodexSettingsPage';
import { setAdapter, type CodexBackendAdapter } from '../src/services/codexClient';
import { useCodexUiStore } from '../src/stores/useCodexUiStore';

function createAdapter(overrides: Partial<CodexBackendAdapter> = {}): CodexBackendAdapter {
  return new Proxy(overrides, {
    get(target, prop: string) {
      if (prop in target) return target[prop as keyof CodexBackendAdapter];
      return async () => {
        if (prop.startsWith('list')) return [];
        if (prop.startsWith('get')) return null;
        if (prop.startsWith('is')) return false;
        return undefined;
      };
    },
  }) as CodexBackendAdapter;
}

afterEach(() => {
  cleanup();
  localStorage.clear();
  useCodexUiStore.setState({
    accounts: [],
    currentAccount: null,
    accountGroups: [],
    localAccessState: null,
  });
  setAdapter(createAdapter());
});

describe('Codex UI interactions', () => {
  it('saves settings through the backend adapter', async () => {
    const saveGeneralConfig = vi.fn(async () => undefined);
    setAdapter(createAdapter({
      getGeneralConfig: async () => ({
        codex_auto_refresh_minutes: 45,
        codex_app_path: '/opt/codex',
        codex_launch_on_switch: true,
        codex_local_access_entry_visible: false,
        codex_auto_switch_primary_threshold: 30,
        codex_quota_alert_threshold: 20,
      }),
      saveGeneralConfig,
    }));

    render(<CodexSettingsPage />);

    await screen.findByDisplayValue('45');
    fireEvent.click(screen.getByRole('button', { name: /save settings/i }));

    await waitFor(() => expect(saveGeneralConfig).toHaveBeenCalledTimes(1));
    expect(saveGeneralConfig).toHaveBeenCalledWith(expect.objectContaining({
      codex_auto_refresh_minutes: 45,
      codex_app_path: '/opt/codex',
      codex_launch_on_switch: true,
      codex_local_access_entry_visible: false,
      codex_auto_switch_primary_threshold: 30,
      codex_quota_alert_threshold: 20,
    }));
  });

  it('offers a switch action for non-current accounts', async () => {
    const switchAccount = vi.fn(async () => ({
      id: 'acct_2',
      email: 'two@example.com',
      auth_mode: 'oauth',
      tokens: { id_token: '', access_token: '' },
      created_at: 1,
      last_used: 2,
    }));
    setAdapter(createAdapter({ switchAccount }));
    useCodexUiStore.setState({
      accounts: [
        { id: 'acct_1', email: 'one@example.com', auth_mode: 'oauth', tokens: { id_token: '', access_token: '' }, created_at: 1, last_used: 2 },
        { id: 'acct_2', email: 'two@example.com', auth_mode: 'oauth', tokens: { id_token: '', access_token: '' }, created_at: 1, last_used: 2 },
      ],
      currentAccount: { id: 'acct_1', email: 'one@example.com', auth_mode: 'oauth', tokens: { id_token: '', access_token: '' }, created_at: 1, last_used: 2 },
      accountGroups: [],
      localAccessState: null,
    });

    render(<CodexAccountsPage />);

    fireEvent.click(screen.getAllByTitle('Switch account')[0]);

    await waitFor(() => expect(switchAccount).toHaveBeenCalledWith('acct_2'));
  });
});
