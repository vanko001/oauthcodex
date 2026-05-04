import { useState, useEffect, useMemo } from 'react';
import {
  Plus, RefreshCw, Download, Trash2, X, Globe, KeyRound,
  Copy, Check, Search, ArrowDownWideNarrow, Tag, Eye, EyeOff,
  FileUp, FolderOpen, FolderPlus, ChevronRight, Server, CircleAlert, Clock, GripVertical,
  LogIn,
} from 'lucide-react';
import { useCodexUiStore } from '../stores/useCodexUiStore';
import { isCodexApiKeyAccount, getCodexQuotaClass, getCodexQuotaWindows, formatResetTime } from '../types/codex';
import { buildCodexAccountPresentation, maskSensitiveValue } from '../presentation/codexPresentation';
import { CodexOverviewTabsHeader, type CodexTab } from '../components/CodexOverviewTabsHeader';
import { CodexLocalAccessModal } from '../components/CodexLocalAccessModal';
import { CodexAccountGroupModal, CodexAddToGroupModal } from '../components/CodexAccountGroupModal';
import { CodexModelProviderManager } from '../components/CodexModelProviderManager';
import { CodexSessionManager } from '../components/CodexSessionManager';
import { CodexWakeupContent } from '../components/CodexWakeupContent';
import { CodexInstancesPage } from './CodexInstancesPage';

const CODEX_OVERVIEW_LAYOUT_MODE_KEY = 'agtools.codex.accounts.overview_layout_mode';
const CODEX_LOCAL_ACCESS_EXPANDED_KEY = 'agtools.codex.local_access_entry_expanded.v1';
const CODEX_CUSTOM_SORT_ORDER_KEY = 'agtools.codex.accounts.custom_sort_order.v1';
const CODEX_PRIVACY_MODE_KEY = 'agtools.codex.accounts.privacy_mode.v1';

type OverviewLayoutMode = 'compact' | 'list' | 'grid';

export function CodexAccountsPage() {
  const store = useCodexUiStore();
  const [activeTab, setActiveTab] = useState<CodexTab>('overview');
  const [searchQuery, setSearchQuery] = useState('');
  const [showAddModal, setShowAddModal] = useState(false);
  const [addTab, setAddTab] = useState<'oauth' | 'apikey' | 'import'>('oauth');
  const [addMessage, setAddMessage] = useState<{ text: string; tone?: string } | null>(null);
  const [tokenInput, setTokenInput] = useState('');
  const [importing, setImporting] = useState(false);
  const [apiKeyInput, setApiKeyInput] = useState('');
  const [apiBaseUrlInput, setApiBaseUrlInput] = useState('');
  const [apiKeyInputVisible, setApiKeyInputVisible] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [layoutMode, setLayoutMode] = useState<OverviewLayoutMode>(() => {
    try { return (localStorage.getItem(CODEX_OVERVIEW_LAYOUT_MODE_KEY) as OverviewLayoutMode) || 'grid'; }
    catch { return 'grid'; }
  });
  const [filterTags, setFilterTags] = useState<string[]>([]);
  const [sortBy, setSortBy] = useState<'email' | 'plan' | 'created' | 'last_used'>('email');
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');
  const [showTagModal, setShowTagModal] = useState(false);
  const [tagEditAccountId, setTagEditAccountId] = useState<string | null>(null);
  const [tagEditValue, setTagEditValue] = useState('');
  const [renamingAccountId, setRenamingAccountId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const [showExportModal, setShowExportModal] = useState(false);
  const [exportJsonContent, setExportJsonContent] = useState('');
  const [exportJsonHidden, setExportJsonHidden] = useState(true);
  const [formattedExportCopied, setFormattedExportCopied] = useState(false);
  const [showGroupModal, setShowGroupModal] = useState(false);
  const [showAddToGroupModal, setShowAddToGroupModal] = useState(false);
  const [localAccessExpanded, setLocalAccessExpanded] = useState(() => {
    try { return localStorage.getItem(CODEX_LOCAL_ACCESS_EXPANDED_KEY) === '1'; }
    catch { return false; }
  });
  const [showLocalAccessModal, setShowLocalAccessModal] = useState(false);
  const [localAccessModalMode, setLocalAccessModalMode] = useState<'panel' | 'members'>('panel');
  const [groupFilter, setGroupFilter] = useState<string[]>([]);
  const [activeGroupId, setActiveGroupId] = useState<string | null>(null);
  const [customSortOrder, setCustomSortOrder] = useState<string[]>(() => {
    try { return JSON.parse(localStorage.getItem(CODEX_CUSTOM_SORT_ORDER_KEY) || '[]'); }
    catch { return []; }
  });
  const [showCustomSortModal, setShowCustomSortModal] = useState(false);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const [oauthUrlCopied, setOauthUrlCopied] = useState(false);
  const [privacyMode, setPrivacyMode] = useState(() => {
    try { return localStorage.getItem(CODEX_PRIVACY_MODE_KEY) === '1'; }
    catch { return false; }
  });

  const maskText = (v?: string | null) => maskSensitiveValue(v, privacyMode);

  useEffect(() => {
    localStorage.setItem(CODEX_OVERVIEW_LAYOUT_MODE_KEY, layoutMode);
  }, [layoutMode]);

  useEffect(() => {
    localStorage.setItem(CODEX_LOCAL_ACCESS_EXPANDED_KEY, localAccessExpanded ? '1' : '0');
  }, [localAccessExpanded]);

  useEffect(() => {
    localStorage.setItem(CODEX_CUSTOM_SORT_ORDER_KEY, JSON.stringify(customSortOrder));
  }, [customSortOrder]);

  useEffect(() => {
    localStorage.setItem(CODEX_PRIVACY_MODE_KEY, privacyMode ? '1' : '0');
  }, [privacyMode]);

  const accounts = store.accounts;
  const currentAccount = store.currentAccount;
  const groups = store.accountGroups;
  const localAccessState = store.localAccessState;

  const allTags = useMemo(() => {
    const tags = new Set<string>();
    accounts.forEach(a => (a.tags || []).forEach(t => { if (t.trim()) tags.add(t.trim()); }));
    return Array.from(tags).sort();
  }, [accounts]);

  const filteredAccounts = useMemo(() => {
    let list = [...accounts];
    const query = searchQuery.trim().toLowerCase();
    if (query) {
      list = list.filter(a => {
        const label = `${a.email || ''} ${a.display_name || ''} ${a.account_name || ''} ${a.api_base_url || ''}`.toLowerCase();
        return label.includes(query);
      });
    }
    if (filterTags.length > 0) {
      list = list.filter(a => (a.tags || []).some(t => filterTags.includes(t.trim())));
    }
    if (activeGroupId) {
      const group = groups.find(g => g.id === activeGroupId);
      if (group) list = list.filter(a => group.accountIds.includes(a.id));
    }
    if (groupFilter.length > 0) {
      list = list.filter(a => groups.some(g => groupFilter.includes(g.id) && g.accountIds.includes(a.id)));
    }
    list.sort((a, b) => {
      if (a.id === currentAccount?.id) return -1;
      if (b.id === currentAccount?.id) return 1;
      const dir = sortDirection === 'asc' ? 1 : -1;
      if (sortBy === 'email') return a.email.localeCompare(b.email) * dir;
      if (sortBy === 'plan') return (a.plan_type || '').localeCompare(b.plan_type || '') * dir;
      if (sortBy === 'created') return (a.created_at - b.created_at) * dir;
      return (a.last_used - b.last_used) * dir;
    });
    if (customSortOrder.length > 0) {
      const sortMap = new Map(customSortOrder.map((id, i) => [id, i]));
      list.sort((a, b) => {
        const ai = sortMap.get(a.id) ?? 999;
        const bi = sortMap.get(b.id) ?? 999;
        return ai - bi;
      });
    }
    return list;
  }, [accounts, searchQuery, filterTags, activeGroupId, groupFilter, groups, sortBy, sortDirection, currentAccount, customSortOrder]);

  const handleDelete = async (id: string) => {
    await store.deleteAccount(id);
    setDeleteConfirmId(null);
  };

  const handleBatchDelete = async () => {
    if (!window.confirm(`Delete ${selectedIds.size} selected account(s)?`)) return;
    await store.deleteAccounts(Array.from(selectedIds));
    setSelectedIds(new Set());
  };

  const handleExport = async () => {
    const ids = selectedIds.size > 0 ? Array.from(selectedIds) : accounts.map(a => a.id);
    const json = await store.exportAccounts(ids);
    setExportJsonContent(json);
    setExportJsonHidden(true);
    setShowExportModal(true);
  };

  const handleSwitchAccount = async (accountId: string) => {
    try {
      await store.switchAccount(accountId);
      setAddMessage({ text: 'Active account changed', tone: 'success' });
      window.setTimeout(() => setAddMessage(null), 1600);
    } catch (e) {
      setAddMessage({ text: String(e), tone: 'error' });
    }
  };

  const toggleSelect = (id: string) => {
    setSelectedIds(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  };

  const toggleSelectAll = () => {
    if (selectedIds.size === filteredAccounts.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(filteredAccounts.map(a => a.id)));
    }
  };

  const handleImportJson = async () => {
    if (!tokenInput.trim()) return;
    setImporting(true);
    try {
      const imported = await store.importFromJson(tokenInput);
      setAddMessage({ text: `Imported ${imported.length} account(s)`, tone: 'success' });
      setTokenInput('');
      setShowAddModal(false);
    } catch (e) {
      setAddMessage({ text: String(e), tone: 'error' });
    } finally {
      setImporting(false);
    }
  };

  const handleStartOAuth = async () => {
    await store.startOAuth();
  };

  const handleCompleteOAuth = async () => {
    await store.completeOAuth();
  };

  const handleCancelOAuth = async () => {
    await store.cancelOAuth();
  };

  const handleSubmitOAuthCallback = async () => {
    if (!tokenInput.trim()) return;
    await store.submitOAuthCallback(tokenInput.trim());
    setTokenInput('');
  };

  const handleAddApiKey = async () => {
    if (!apiKeyInput.trim()) return;
    try {
      await store.addApiKeyAccount(apiKeyInput.trim(), apiBaseUrlInput.trim() || undefined);
      setApiKeyInput('');
      setApiBaseUrlInput('');
      setShowAddModal(false);
      setAddMessage({ text: 'API Key account added', tone: 'success' });
    } catch (e) {
      setAddMessage({ text: String(e), tone: 'error' });
    }
  };

  const renderQuotaDisplay = (account: any) => {
    const quota = account.quota;
    if (!quota) return <span className="quota-empty">No quota data</span>;
    const windows = getCodexQuotaWindows(quota);
    return (
      <div className="quota-bars">
        {windows.map(w => (
          <div key={w.id} className="quota-bar-row" title={`${w.label}: ${w.percentage}% ${formatResetTime(w.resetTime)}`}>
            <span className="quota-bar-label">{w.label}</span>
            <div className="quota-bar-track">
              <div className={`quota-bar-fill ${getCodexQuotaClass(w.percentage)}`} style={{ width: `${w.percentage}%` }} />
            </div>
            <span className={`quota-bar-value ${getCodexQuotaClass(w.percentage)}`}>{w.percentage}%</span>
          </div>
        ))}
      </div>
    );
  };

  const renderAccountCard = (account: any) => {
    const presentation = buildCodexAccountPresentation(account);
    const isCurrent = account.id === currentAccount?.id;
    const isSelected = selectedIds.has(account.id);
    const isApiKey = isCodexApiKeyAccount(account);
    const editing = renamingAccountId === account.id;
    const accountLabel = account.account_name || account.display_name || account.email || account.id;
    return (
      <div key={account.id} className={`account-card ${layoutMode} ${isCurrent ? 'current' : ''} ${isSelected ? 'selected' : ''}`}>
        <div className="account-card-header">
          <input type="checkbox" checked={isSelected} onChange={() => toggleSelect(account.id)} className="account-select" />
          <div className="account-email-area">
            {editing ? (
              <input className="inline-rename-input" value={renameValue} onChange={e => setRenameValue(e.target.value)}
                onKeyDown={e => { if (e.key === 'Enter') { void store.updateAccountName(account.id, renameValue); setRenamingAccountId(null); } if (e.key === 'Escape') setRenamingAccountId(null); }}
                onBlur={() => { if (renameValue.trim()) void store.updateAccountName(account.id, renameValue); setRenamingAccountId(null); }}
                autoFocus />
            ) : (
              <span className="account-email" onDoubleClick={() => { setRenamingAccountId(account.id); setRenameValue(accountLabel); }} title="Double-click to rename">
                {maskText(accountLabel)}
              </span>
            )}
            {isCurrent && <span className="current-badge">Current</span>}
            {isApiKey && <span className="api-key-badge">API Key</span>}
          </div>
          <div className="account-card-actions">
            {!isCurrent && (
              <button className="icon-btn" onClick={() => void handleSwitchAccount(account.id)} title="Switch account"><LogIn size={14} /></button>
            )}
            {!isApiKey && (
              <button className="icon-btn" onClick={() => store.refreshQuota(account.id)} title="Refresh quota"><RefreshCw size={14} /></button>
            )}
            <button className="icon-btn" onClick={() => { setTagEditAccountId(account.id); setTagEditValue((account.tags || []).join(', ')); setShowTagModal(true); }} title="Edit tags"><Tag size={14} /></button>
            <button className="icon-btn danger" onClick={() => setDeleteConfirmId(account.id)} title="Delete"><Trash2 size={14} /></button>
          </div>
        </div>
        <div className="account-card-body">
          <div className="account-plan-section">
            <span className={`plan-badge ${presentation.planClass}`}>{presentation.planLabel}</span>
            {isApiKey && account.api_base_url && <code className="api-base-url">{account.api_base_url}</code>}
            {account.quota_error && (
              <span className="quota-error-indicator" title={account.quota_error.message}>
                <CircleAlert size={14} /> Error
              </span>
            )}
          </div>
          {account.tags && account.tags.length > 0 && (
            <div className="account-tags">
              {account.tags.map((t: string) => <span key={t} className="tag-chip">{t}</span>)}
            </div>
          )}
          {renderQuotaDisplay(account)}
          <div className="account-meta">
            <span className="meta-item"><Clock size={12} /> {new Date(account.last_used * 1000).toLocaleDateString()}</span>
            {!isApiKey && account.quota?.hourly_reset_time && (
              <span className="meta-item">Reset: {formatResetTime(account.quota.hourly_reset_time)}</span>
            )}
          </div>
        </div>
        {deleteConfirmId === account.id && (
          <div className="delete-confirm-overlay">
            <span>Delete this account?</span>
            <div className="delete-confirm-actions">
              <button className="btn btn-danger btn-sm" onClick={() => handleDelete(account.id)}>Delete</button>
              <button className="btn btn-secondary btn-sm" onClick={() => setDeleteConfirmId(null)}>Cancel</button>
            </div>
          </div>
        )}
      </div>
    );
  };

  const groupFolders = useMemo(() => {
    if (activeGroupId) {
      const g = groups.find(grp => grp.id === activeGroupId);
      if (!g) return [];
      return [{ group: g, accounts: filteredAccounts.filter(a => g.accountIds.includes(a.id)) }];
    }
    return groups.map(g => ({
      group: g,
      accounts: filteredAccounts.filter(a => g.accountIds.includes(a.id)),
    }));
  }, [groups, filteredAccounts, activeGroupId]);

  const ungroupedAccounts = useMemo(() => {
    if (activeGroupId) return [];
    const groupedIds = new Set(groups.flatMap(g => g.accountIds));
    return filteredAccounts.filter(a => !groupedIds.has(a.id));
  }, [groups, filteredAccounts, activeGroupId]);

  const oauthAccountIds = useMemo(() => accounts.filter(a => !isCodexApiKeyAccount(a)).map(a => a.id), [accounts]);

  return (
    <div className="codex-accounts-page">
      <div className="page-header">
        <h1>Codex Accounts</h1>
        <div className="page-header-stats">
          <span>{accounts.length} accounts</span>
          {currentAccount && <span className="current-label">Active: {maskText(currentAccount.account_name || currentAccount.display_name || currentAccount.email)}</span>}
        </div>
      </div>

      <CodexOverviewTabsHeader active={activeTab} onTabChange={setActiveTab} />
      <div className="page-toolbar">
        <div className="toolbar-left">
          <div className="search-box">
            <Search size={16} className="search-icon" />
            <input type="text" placeholder="Search accounts..." value={searchQuery} onChange={e => setSearchQuery(e.target.value)} />
          </div>
          <div className="filter-chips">
            {allTags.map(tag => (
              <button key={tag} className={`filter-chip ${filterTags.includes(tag) ? 'active' : ''}`}
                onClick={() => setFilterTags(prev => prev.includes(tag) ? prev.filter(t => t !== tag) : [...prev, tag])}>
                {tag}
              </button>
            ))}
          </div>
        </div>
        <div className="toolbar-right">
          <select className="form-select sort-select" value={sortBy} onChange={e => setSortBy(e.target.value as any)}>
            <option value="email">Email</option>
            <option value="plan">Plan</option>
            <option value="created">Created</option>
            <option value="last_used">Last Used</option>
          </select>
          <button className="icon-btn" onClick={() => setSortDirection(d => d === 'asc' ? 'desc' : 'asc')} title="Toggle sort direction">
            <ArrowDownWideNarrow size={16} />
          </button>
          <button className={`icon-btn ${privacyMode ? 'active' : ''}`} onClick={() => setPrivacyMode(v => !v)} title={privacyMode ? 'Show account text' : 'Hide account text'}>
            {privacyMode ? <EyeOff size={16} /> : <Eye size={16} />}
          </button>
          <div className="layout-toggle">
            {(['grid', 'list', 'compact'] as const).map(mode => (
              <button key={mode} className={`icon-btn ${layoutMode === mode ? 'active' : ''}`}
                onClick={() => setLayoutMode(mode)}>
                {mode === 'grid' ? '\u25A6' : mode === 'list' ? '\u2630' : '\u2261'}
              </button>
            ))}
          </div>
          <button className="btn btn-secondary btn-sm" onClick={() => setShowGroupModal(true)}><FolderPlus size={14} /> Groups</button>
          <button className="btn btn-secondary btn-sm" onClick={() => setShowCustomSortModal(true)}><GripVertical size={14} /> Sort</button>
          <button className="btn btn-secondary btn-sm" onClick={handleExport}><Download size={14} /> Export</button>
          <button className="btn btn-primary btn-sm" onClick={() => { setShowAddModal(true); setAddTab('oauth'); }}>
            <Plus size={14} /> Add Account
          </button>
        </div>
      </div>

      {addMessage && (
        <div className={`message-bar ${addMessage.tone === 'error' ? 'error' : 'success'}`}>
          {addMessage.text}
          <button className="message-close" onClick={() => setAddMessage(null)}><X size={14} /></button>
        </div>
      )}

      <div className="local-access-section" onClick={() => setLocalAccessExpanded(!localAccessExpanded)}>
        <div className="local-access-header">
          <Server size={16} />
          <span>Local API Service</span>
          <ChevronRight size={14} className={`chevron ${localAccessExpanded ? 'expanded' : ''}`} />
          <span className={`status-badge ${localAccessState?.running ? 'running' : 'stopped'}`}>
            {localAccessState?.running ? 'Running' : 'Stopped'}
          </span>
        </div>
        {localAccessExpanded && (
          <div className="local-access-preview">
            <button className="btn btn-secondary btn-sm" onClick={(e) => { e.stopPropagation(); setLocalAccessModalMode('panel'); setShowLocalAccessModal(true); }}>
              <Server size={14} /> Manage Service
            </button>
            <button className="btn btn-secondary btn-sm" onClick={(e) => { e.stopPropagation(); setLocalAccessModalMode('members'); setShowLocalAccessModal(true); }}>
              <FolderPlus size={14} /> Manage Members
            </button>
          </div>
        )}
      </div>

      {activeTab === 'overview' && (
        <div className="accounts-container">
          <div className="accounts-toolbar">
            <label className="select-all">
              <input type="checkbox"
                checked={selectedIds.size === filteredAccounts.length && filteredAccounts.length > 0}
                onChange={toggleSelectAll} />
              Select All ({filteredAccounts.length})
            </label>
            {selectedIds.size > 0 && (
              <div className="batch-actions">
                <button className="btn btn-danger btn-sm" onClick={handleBatchDelete}>
                  <Trash2 size={14} /> Delete ({selectedIds.size})
                </button>
                <button className="btn btn-secondary btn-sm" onClick={() => setShowAddToGroupModal(true)}>
                  <FolderPlus size={14} /> Add to Group ({selectedIds.size})
                </button>
              </div>
            )}
          </div>

          {groups.length > 0 && !activeGroupId && (
            <div className="group-folders">
              {groupFolders.map(f => f.accounts.length > 0 && (
                <div key={f.group.id} className="group-folder">
                  <div className="group-folder-header" onClick={() => setActiveGroupId(f.group.id)}>
                    <FolderOpen size={16} />
                    <span className="group-folder-name">{f.group.name}</span>
                    <span className="group-folder-count">{f.accounts.length}</span>
                  </div>
                  <div className={`accounts-grid ${layoutMode}`}>
                    {f.accounts.map(renderAccountCard)}
                  </div>
                </div>
              ))}
            </div>
          )}

          {activeGroupId && (
            <div className="group-folder-active">
              <div className="group-folder-header">
                <button className="icon-btn" onClick={() => setActiveGroupId(null)}><X size={14} /></button>
                <FolderOpen size={16} />
                <span className="group-folder-name">{groups.find(g => g.id === activeGroupId)?.name}</span>
              </div>
              <div className={`accounts-grid ${layoutMode}`}>
                {filteredAccounts.map(renderAccountCard)}
              </div>
            </div>
          )}

          <div className={`accounts-grid ${layoutMode}`}>
            {!activeGroupId && ungroupedAccounts.map(renderAccountCard)}
          </div>

          {filteredAccounts.length === 0 && (
            <div className="empty-state">
              <h3>No accounts found</h3>
              <p>Add an account via OAuth, API Key, or import from JSON.</p>
            </div>
          )}
        </div>
      )}

      {activeTab === 'providers' && (
        <CodexModelProviderManager
          onProvidersChanged={() => {}}
        />
      )}

      {activeTab === 'wakeup' && (
        <CodexWakeupContent accounts={accounts} onRefreshAccounts={store.fetchAccounts} />
      )}

      {activeTab === 'instances' && (
        <CodexInstancesPage />
      )}

      {activeTab === 'sessions' && (
        <CodexSessionManager />
      )}

      {showAddModal && (
        <div className="modal-overlay" onClick={() => setShowAddModal(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Add Codex Account</h2>
              <button className="modal-close" onClick={() => setShowAddModal(false)}><X /></button>
            </div>
            <div className="modal-body">
              <div className="add-tabs">
                <button className={`add-tab ${addTab === 'oauth' ? 'active' : ''}`} onClick={() => setAddTab('oauth')}>OAuth</button>
                <button className={`add-tab ${addTab === 'apikey' ? 'active' : ''}`} onClick={() => setAddTab('apikey')}>API Key</button>
                <button className={`add-tab ${addTab === 'import' ? 'active' : ''}`} onClick={() => setAddTab('import')}>Import</button>
              </div>

              {addTab === 'oauth' && (
                <div className="oauth-flow">
                  {store.oauthState === 'idle' && (
                    <div className="oauth-start">
                      <p>Start OAuth login to add your ChatGPT account.</p>
                      <button className="btn btn-primary" onClick={handleStartOAuth}>
                        <Globe size={16} /> Start OAuth Login
                      </button>
                      {store.oauthError && <div className="form-error">{store.oauthError}</div>}
                    </div>
                  )}
                  {store.oauthState === 'waiting' && store.oauthAuthUrl && (
                    <div className="oauth-waiting">
                      <p>Authorization URL ready. Open in browser or paste callback:</p>
                      <code className="oauth-url">{store.oauthAuthUrl}</code>
                      <div className="oauth-actions">
                        <button className="btn btn-primary btn-sm" onClick={() => { navigator.clipboard.writeText(store.oauthAuthUrl || ''); setOauthUrlCopied(true); window.setTimeout(() => setOauthUrlCopied(false), 1200); }}>
                          {oauthUrlCopied ? <Check size={14} /> : <Copy size={14} />} Copy URL
                        </button>
                        <button className="btn btn-secondary btn-sm" onClick={handleCompleteOAuth}>Complete</button>
                        <button className="btn btn-danger btn-sm" onClick={handleCancelOAuth}>Cancel</button>
                      </div>
                      <div className="oauth-manual-callback">
                        <label>Or paste callback URL:</label>
                        <div className="oauth-callback-row">
                          <input type="text" className="form-input" placeholder="http://localhost:1455/auth/callback?code=..." value={tokenInput} onChange={e => setTokenInput(e.target.value)} />
                          <button className="btn btn-primary btn-sm" onClick={handleSubmitOAuthCallback} disabled={!tokenInput.trim()}>Submit</button>
                        </div>
                      </div>
                    </div>
                  )}
                  {store.oauthState === 'preparing' && <p>Preparing OAuth...</p>}
                  {store.oauthState === 'completing' && <p>Completing OAuth...</p>}
                </div>
              )}

              {addTab === 'apikey' && (
                <div className="apikey-form">
                  <div className="form-group">
                    <label>API Key</label>
                    <div className="api-key-input-row">
                      <input type={apiKeyInputVisible ? 'text' : 'password'} className="form-input"
                        value={apiKeyInput} onChange={e => setApiKeyInput(e.target.value)} placeholder="sk-..." />
                      <button className="icon-btn" onClick={() => setApiKeyInputVisible(!apiKeyInputVisible)}>
                        {apiKeyInputVisible ? <EyeOff size={14} /> : <Eye size={14} />}
                      </button>
                    </div>
                  </div>
                  <div className="form-group">
                    <label>Base URL (optional, defaults to OpenAI)</label>
                    <input type="text" className="form-input" value={apiBaseUrlInput}
                      onChange={e => setApiBaseUrlInput(e.target.value)} placeholder="https://api.openai.com/v1" />
                  </div>
                  <button className="btn btn-primary" onClick={handleAddApiKey} disabled={!apiKeyInput.trim()}>
                    <KeyRound size={14} /> Add API Key Account
                  </button>
                </div>
              )}

              {addTab === 'import' && (
                <div className="import-form">
                  <div className="form-group">
                    <label>Paste JSON content (single account or array)</label>
                    <textarea className="form-textarea" value={tokenInput} onChange={e => setTokenInput(e.target.value)}
                      placeholder='{"tokens":{"id_token":"...","access_token":"..."}}' rows={6} />
                  </div>
                  <button className="btn btn-primary" onClick={handleImportJson} disabled={importing || !tokenInput.trim()}>
                    <FileUp size={14} /> {importing ? 'Importing...' : 'Import'}
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {showExportModal && (
        <div className="modal-overlay" onClick={() => setShowExportModal(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Export Accounts</h2>
              <button className="modal-close" onClick={() => setShowExportModal(false)}><X /></button>
            </div>
            <div className="modal-body">
              <div className="export-actions">
                <button className="btn btn-secondary btn-sm" onClick={() => setExportJsonHidden(!exportJsonHidden)}>
                  {exportJsonHidden ? <Eye size={14} /> : <EyeOff size={14} />} {exportJsonHidden ? 'Show' : 'Hide'}
                </button>
                <button className="btn btn-primary btn-sm" onClick={async () => {
                  try {
                    await navigator.clipboard.writeText(exportJsonContent);
                    setFormattedExportCopied(true);
                    window.setTimeout(() => setFormattedExportCopied(false), 1200);
                  } catch {}
                }}>
                  {formattedExportCopied ? <Check size={14} /> : <Copy size={14} />} Copy
                </button>
              </div>
              <textarea className="export-textarea" readOnly value={exportJsonHidden ? '*** hidden ***' : exportJsonContent} rows={12} />
            </div>
          </div>
        </div>
      )}

      {showTagModal && tagEditAccountId && (
        <div className="modal-overlay" onClick={() => setShowTagModal(false)}>
          <div className="modal modal-sm" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Edit Tags</h2>
              <button className="modal-close" onClick={() => setShowTagModal(false)}><X /></button>
            </div>
            <div className="modal-body">
              <div className="form-group">
                <label>Tags (comma separated)</label>
                <input type="text" className="form-input" value={tagEditValue}
                  onChange={e => setTagEditValue(e.target.value)} placeholder="tag1, tag2" />
              </div>
              <button className="btn btn-primary" onClick={async () => {
                const tags = tagEditValue.split(',').map(t => t.trim()).filter(Boolean);
                await store.updateAccountTags(tagEditAccountId, tags);
                setShowTagModal(false);
              }}>Save Tags</button>
            </div>
          </div>
        </div>
      )}

      <CodexAccountGroupModal
        isOpen={showGroupModal}
        onClose={() => setShowGroupModal(false)}
        onGroupsChanged={store.fetchAccountGroups}
        groupFilter={groupFilter}
        onToggleGroupFilter={(id) => setGroupFilter(prev => prev.includes(id) ? prev.filter(x => x !== id) : [...prev, id])}
        onClearGroupFilter={() => setGroupFilter([])}
      />

      <CodexAddToGroupModal
        isOpen={showAddToGroupModal}
        onClose={() => setShowAddToGroupModal(false)}
        accountIds={Array.from(selectedIds)}
        onAdded={() => { setSelectedIds(new Set()); store.fetchAccountGroups(); }}
      />

      <CodexLocalAccessModal
        isOpen={showLocalAccessModal}
        mode={localAccessModalMode}
        state={localAccessState}
        accounts={accounts}
        accountGroups={groups}
        initialSelectedIds={localAccessState?.collection?.accountIds || oauthAccountIds}
        maskAccountText={maskText}
        onClose={() => setShowLocalAccessModal(false)}
        onSaveAccounts={async (payload) => {
          await store.saveLocalAccessAccounts(payload.accountIds, payload.restrictFreeAccounts);
        }}
        onClearStats={store.clearLocalAccessStats}
        onRefreshStats={store.fetchLocalAccessState}
        onUpdatePort={store.updateLocalAccessPort}
        onUpdateRoutingStrategy={async (strategy: any) => {
          await store.updateLocalAccessRouting(strategy);
        }}
        onRotateApiKey={store.rotateLocalAccessKey}
        onKillPort={store.killLocalAccessPort}
        onToggleEnabled={async () => {
          await store.setLocalAccessEnabled(!localAccessState?.collection?.enabled);
        }}
        onTest={async () => { await store.prepareLocalAccessForRestart(); return 0; }}
        saving={false}
        testing={false}
        starting={false}
        portCleanupBusy={false}
      />

      {showCustomSortModal && (
        <div className="modal-overlay" onClick={() => setShowCustomSortModal(false)}>
          <div className="modal modal-sm" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Custom Sort Order</h2>
              <button className="modal-close" onClick={() => setShowCustomSortModal(false)}><X /></button>
            </div>
            <div className="modal-body">
              <p>Drag to reorder accounts (custom sort takes precedence).</p>
              <div className="custom-sort-list">
                {customSortOrder.map((id, i) => {
                  const account = accounts.find(a => a.id === id);
                  if (!account) return null;
                  return (
                    <div key={id} className="custom-sort-item">
                      <GripVertical size={14} className="drag-handle" />
                      <span className="custom-sort-email">{maskText(account.email)}</span>
                      <button className="icon-btn" onClick={() => {
                        const next = [...customSortOrder];
                        if (i > 0) { [next[i], next[i - 1]] = [next[i - 1], next[i]]; setCustomSortOrder(next); }
                      }}><ChevronRight size={14} style={{ transform: 'rotate(-90deg)' }} /></button>
                      <button className="icon-btn" onClick={() => {
                        const next = [...customSortOrder];
                        if (i < next.length - 1) { [next[i], next[i + 1]] = [next[i + 1], next[i]]; setCustomSortOrder(next); }
                      }}><ChevronRight size={14} style={{ transform: 'rotate(90deg)' }} /></button>
                    </div>
                  );
                })}
              </div>
              <button className="btn btn-primary btn-sm" onClick={() => setShowCustomSortModal(false)}>Done</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
