import { create } from 'zustand';
import type { CodexAccount, CodexQuota } from '../types/codex';
import type { CodexLocalAccessState } from '../types/codexLocalAccess';
import type { CodexCliStatus, CodexWakeupState, CodexWakeupHistoryItem } from '../types/codexWakeup';
import type { InstanceProfile } from '../types/instance';
import { codexClient } from '../services/codexClient';
import type { CodexPage } from '../types/navigation';

export interface CodexAccountGroup {
  id: string;
  name: string;
  accountIds: string[];
}

export interface CodexModelProvider {
  id: string;
  name: string;
  baseUrl: string;
  website?: string;
  apiKeyUrl?: string;
  apiKeys: CodexModelProviderApiKey[];
}

export interface CodexModelProviderApiKey {
  id: string;
  name?: string;
  apiKey: string;
}

interface CodexUiStoreState {
  currentPage: CodexPage;
  setCurrentPage: (page: CodexPage) => void;

  accounts: CodexAccount[];
  currentAccount: CodexAccount | null;
  loading: boolean;
  error: string | null;
  fetchAccounts: () => Promise<void>;
  fetchCurrentAccount: () => Promise<void>;
  switchAccount: (accountId: string) => Promise<CodexAccount>;
  deleteAccount: (accountId: string) => Promise<void>;
  deleteAccounts: (accountIds: string[]) => Promise<void>;
  refreshQuota: (accountId: string) => Promise<CodexQuota>;
  refreshAllQuotas: () => Promise<number>;
  importFromJson: (jsonContent: string) => Promise<CodexAccount[]>;
  exportAccounts: (accountIds: string[]) => Promise<string>;
  updateAccountName: (accountId: string, name: string) => Promise<CodexAccount>;
  updateAccountTags: (accountId: string, tags: string[]) => Promise<CodexAccount>;

  oauthState: 'idle' | 'preparing' | 'waiting' | 'completing';
  oauthLoginId: string | null;
  oauthAuthUrl: string | null;
  oauthError: string | null;
  startOAuth: () => Promise<void>;
  completeOAuth: () => Promise<void>;
  cancelOAuth: () => Promise<void>;
  submitOAuthCallback: (url: string) => Promise<void>;
  addApiKeyAccount: (apiKey: string, apiBaseUrl?: string, providerMode?: string, providerId?: string, providerName?: string) => Promise<CodexAccount>;

  accountGroups: CodexAccountGroup[];
  fetchAccountGroups: () => Promise<void>;
  createAccountGroup: (name: string) => Promise<CodexAccountGroup>;
  renameAccountGroup: (id: string, name: string) => Promise<void>;
  deleteAccountGroup: (id: string) => Promise<void>;
  assignToGroup: (groupId: string, accountIds: string[]) => Promise<void>;
  removeFromGroup: (groupId: string, accountIds: string[]) => Promise<void>;

  modelProviders: CodexModelProvider[];
  fetchModelProviders: () => Promise<void>;
  createModelProvider: (payload: { name: string; baseUrl: string; website?: string; apiKeyUrl?: string; initialApiKey?: string; initialApiKeyName?: string }) => Promise<CodexModelProvider>;
  updateModelProvider: (id: string, payload: { name?: string; baseUrl?: string; website?: string; apiKeyUrl?: string }) => Promise<void>;
  deleteModelProvider: (id: string) => Promise<void>;
  addModelProviderApiKey: (providerId: string, apiKey: string, name?: string) => Promise<void>;
  removeModelProviderApiKey: (providerId: string, apiKeyId: string) => Promise<void>;

  localAccessState: CodexLocalAccessState | null;
  fetchLocalAccessState: () => Promise<void>;
  saveLocalAccessAccounts: (accountIds: string[], restrictFreeAccounts: boolean) => Promise<void>;
  updateLocalAccessPort: (port: number) => Promise<void>;
  updateLocalAccessRouting: (strategy: string) => Promise<void>;
  rotateLocalAccessKey: () => Promise<void>;
  clearLocalAccessStats: () => Promise<void>;
  setLocalAccessEnabled: (enabled: boolean) => Promise<void>;
  prepareLocalAccessForRestart: () => Promise<void>;
  killLocalAccessPort: () => Promise<void>;

  instances: InstanceProfile[];
  fetchInstances: () => Promise<void>;
  createInstance: (payload: { name: string; userDataDir: string; copySourceInstanceId: string; bindAccountId?: string | null; launchMode?: string }) => Promise<InstanceProfile>;
  updateInstance: (payload: { instanceId: string; name?: string; bindAccountId?: string | null; launchMode?: string }) => Promise<InstanceProfile>;
  deleteInstance: (instanceId: string) => Promise<void>;
  startInstance: (instanceId: string) => Promise<InstanceProfile>;
  stopInstance: (instanceId: string) => Promise<InstanceProfile>;

  wakeupRuntime: CodexCliStatus | null;
  wakeupState: CodexWakeupState;
  wakeupHistory: CodexWakeupHistoryItem[];
  fetchWakeup: () => Promise<void>;
  saveWakeupState: (enabled: boolean, tasks: any[], presets: any[]) => Promise<void>;
  runWakeupTask: (taskId: string) => Promise<void>;
  clearWakeupHistory: () => Promise<void>;
}

export const useCodexUiStore = create<CodexUiStoreState>((set, get) => ({
  currentPage: 'accounts',
  setCurrentPage: (page) => set({ currentPage: page }),

  accounts: [],
  currentAccount: null,
  loading: false,
  error: null,

  fetchAccounts: async () => {
    set({ loading: true, error: null });
    try {
      const accounts = await codexClient.listAccounts();
      set({ accounts, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  fetchCurrentAccount: async () => {
    try {
      const currentAccount = await codexClient.getCurrentAccount();
      set({ currentAccount });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  switchAccount: async (accountId) => {
    const account = await codexClient.switchAccount(accountId);
    set({ currentAccount: account });
    await get().fetchAccounts();
    return account;
  },
  deleteAccount: async (accountId) => {
    await codexClient.deleteAccount(accountId);
    await get().fetchAccounts();
    await get().fetchCurrentAccount();
  },
  deleteAccounts: async (accountIds) => {
    await codexClient.deleteAccounts(accountIds);
    await get().fetchAccounts();
    await get().fetchCurrentAccount();
  },
  refreshQuota: async (accountId) => {
    const quota = await codexClient.refreshQuota(accountId);
    await get().fetchAccounts();
    return quota;
  },
  refreshAllQuotas: async () => {
    const count = await codexClient.refreshAllQuotas();
    await get().fetchAccounts();
    return count;
  },
  importFromJson: async (jsonContent) => {
    const accounts = await codexClient.importFromJson(jsonContent);
    await get().fetchAccounts();
    return accounts;
  },
  exportAccounts: async (accountIds) => {
    return codexClient.exportAccounts(accountIds);
  },
  updateAccountName: async (accountId, name) => {
    const account = await codexClient.updateAccountName(accountId, name);
    await get().fetchAccounts();
    return account;
  },
  updateAccountTags: async (accountId, tags) => {
    const account = await codexClient.updateAccountTags(accountId, tags);
    await get().fetchAccounts();
    return account;
  },

  oauthState: 'idle',
  oauthLoginId: null,
  oauthAuthUrl: null,
  oauthError: null,
  startOAuth: async () => {
    set({ oauthState: 'preparing', oauthError: null });
    try {
      const resp = await codexClient.startOAuthLogin();
      set({ oauthState: 'waiting', oauthLoginId: resp.loginId, oauthAuthUrl: resp.authUrl });
    } catch (e) {
      set({ oauthState: 'idle', oauthError: String(e) });
    }
  },
  completeOAuth: async () => {
    const { oauthLoginId } = get();
    if (!oauthLoginId) return;
    set({ oauthState: 'completing' });
    try {
      await codexClient.completeOAuthLogin(oauthLoginId);
      set({ oauthState: 'idle', oauthLoginId: null, oauthAuthUrl: null });
      await get().fetchAccounts();
    } catch (e) {
      set({ oauthState: 'idle', oauthError: String(e), oauthLoginId: null, oauthAuthUrl: null });
    }
  },
  cancelOAuth: async () => {
    const { oauthLoginId } = get();
    if (oauthLoginId) await codexClient.cancelOAuthLogin(oauthLoginId);
    set({ oauthState: 'idle', oauthLoginId: null, oauthAuthUrl: null, oauthError: null });
  },
  submitOAuthCallback: async (url) => {
    const { oauthLoginId } = get();
    if (!oauthLoginId) return;
    set({ oauthState: 'completing' });
    try {
      await codexClient.submitOAuthCallbackUrl(oauthLoginId, url);
      await codexClient.completeOAuthLogin(oauthLoginId);
      set({ oauthState: 'idle', oauthLoginId: null, oauthAuthUrl: null });
      await get().fetchAccounts();
    } catch (e) {
      set({ oauthState: 'idle', oauthError: String(e), oauthLoginId: null, oauthAuthUrl: null });
    }
  },
  addApiKeyAccount: async (apiKey, apiBaseUrl, providerMode, providerId, providerName) => {
    const account = await codexClient.addAccountWithApiKey(apiKey, apiBaseUrl, providerMode as any, providerId, providerName);
    await get().fetchAccounts();
    return account;
  },

  accountGroups: [],
  fetchAccountGroups: async () => {
    const groups = await codexClient.listCodexAccountGroups();
    set({ accountGroups: groups });
  },
  createAccountGroup: async (name) => {
    const group = await codexClient.createCodexGroup(name);
    await get().fetchAccountGroups();
    return group;
  },
  renameAccountGroup: async (id, name) => {
    await codexClient.renameCodexGroup(id, name);
    await get().fetchAccountGroups();
  },
  deleteAccountGroup: async (id) => {
    await codexClient.deleteCodexGroup(id);
    await get().fetchAccountGroups();
  },
  assignToGroup: async (groupId, accountIds) => {
    await codexClient.assignAccountsToCodexGroup(groupId, accountIds);
    await get().fetchAccountGroups();
  },
  removeFromGroup: async (groupId, accountIds) => {
    await codexClient.removeAccountsFromCodexGroup(groupId, accountIds);
    await get().fetchAccountGroups();
  },

  modelProviders: [],
  fetchModelProviders: async () => {
    const providers = await codexClient.listCodexModelProviders();
    set({ modelProviders: providers });
  },
  createModelProvider: async (payload) => {
    const result = await codexClient.createCodexModelProvider(payload);
    await get().fetchModelProviders();
    return get().modelProviders.find(provider => provider.id === result.id) ?? {
      id: result.id,
      name: payload.name,
      baseUrl: payload.baseUrl,
      website: payload.website,
      apiKeyUrl: payload.apiKeyUrl,
      apiKeys: payload.initialApiKey ? [{ id: `cmk_${Date.now()}`, name: payload.initialApiKeyName, apiKey: payload.initialApiKey }] : [],
    };
  },
  updateModelProvider: async (id, payload) => {
    await codexClient.updateCodexModelProvider(id, payload);
    await get().fetchModelProviders();
  },
  deleteModelProvider: async (id) => {
    await codexClient.deleteCodexModelProvider(id);
    await get().fetchModelProviders();
  },
  addModelProviderApiKey: async (providerId, apiKey, name) => {
    await codexClient.addApiKeyToCodexModelProvider(providerId, apiKey, name);
    await get().fetchModelProviders();
  },
  removeModelProviderApiKey: async (providerId, apiKeyId) => {
    await codexClient.removeApiKeyFromCodexModelProvider(providerId, apiKeyId);
    await get().fetchModelProviders();
  },

  localAccessState: null,
  fetchLocalAccessState: async () => {
    const state = await codexClient.getLocalAccessState();
    set({ localAccessState: state });
  },
  saveLocalAccessAccounts: async (accountIds, restrictFreeAccounts) => {
    await codexClient.saveLocalAccessAccounts(accountIds, restrictFreeAccounts);
    await get().fetchLocalAccessState();
  },
  updateLocalAccessPort: async (port) => {
    await codexClient.updateLocalAccessPort(port);
    await get().fetchLocalAccessState();
  },
  updateLocalAccessRouting: async (strategy) => {
    await codexClient.updateLocalAccessRoutingStrategy(strategy as any);
    await get().fetchLocalAccessState();
  },
  rotateLocalAccessKey: async () => {
    await codexClient.rotateLocalAccessApiKey();
    await get().fetchLocalAccessState();
  },
  clearLocalAccessStats: async () => {
    await codexClient.clearLocalAccessStats();
    await get().fetchLocalAccessState();
  },
  setLocalAccessEnabled: async (enabled) => {
    await codexClient.setLocalAccessEnabled(enabled);
    await get().fetchLocalAccessState();
  },
  prepareLocalAccessForRestart: async () => {
    const state = await codexClient.prepareLocalAccessForRestart();
    set({ localAccessState: state });
  },
  killLocalAccessPort: async () => {
    await codexClient.killLocalAccessPort();
    await get().fetchLocalAccessState();
  },

  instances: [],
  fetchInstances: async () => {
    const instances = await codexClient.listInstances();
    set({ instances });
  },
  createInstance: async (payload) => {
    const instance = await codexClient.createInstance(payload as any);
    await get().fetchInstances();
    return instance;
  },
  updateInstance: async (payload) => {
    const instance = await codexClient.updateInstance(payload as any);
    await get().fetchInstances();
    return instance;
  },
  deleteInstance: async (instanceId) => {
    await codexClient.deleteInstance(instanceId);
    await get().fetchInstances();
  },
  startInstance: async (instanceId) => {
    const instance = await codexClient.startInstance(instanceId);
    await get().fetchInstances();
    return instance;
  },
  stopInstance: async (instanceId) => {
    const instance = await codexClient.stopInstance(instanceId);
    await get().fetchInstances();
    return instance;
  },

  wakeupRuntime: null,
  wakeupState: { enabled: false, tasks: [], model_presets: [], model_preset_migrations: [] },
  wakeupHistory: [],
  fetchWakeup: async () => {
    try {
      const overview = await codexClient.getWakeupOverview();
      set({ wakeupRuntime: overview.runtime, wakeupState: overview.state, wakeupHistory: overview.history });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  saveWakeupState: async (enabled, tasks, presets) => {
    const state = await codexClient.saveWakeupState(enabled, tasks, presets);
    set({ wakeupState: state });
  },
  runWakeupTask: async (taskId) => {
    await codexClient.runWakeupTask(taskId, `run_${Date.now()}`);
    await get().fetchWakeup();
  },
  clearWakeupHistory: async () => {
    await codexClient.clearWakeupHistory();
    set({ wakeupHistory: [] });
  },
}));
