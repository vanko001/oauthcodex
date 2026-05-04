import { useState, useEffect, useMemo } from 'react';
import { Check, CircleAlert, Copy, Eye, EyeOff, Gauge, Power, RefreshCw, Search, Server, Trash2, Wrench, X } from 'lucide-react';
import type { CodexAccount } from '../types/codex';
import type { CodexAccountGroup } from '../stores/useCodexUiStore';
import type { CodexLocalAccessRoutingStrategy, CodexLocalAccessState } from '../types/codexLocalAccess';
import { isCodexApiKeyAccount, getCodexPlanFilterKey } from '../types/codex';
import { buildCodexAccountPresentation } from '../presentation/codexPresentation';

interface CodexLocalAccessModalProps {
  isOpen: boolean;
  mode: 'panel' | 'members';
  state: CodexLocalAccessState | null;
  accounts: CodexAccount[];
  accountGroups: CodexAccountGroup[];
  initialSelectedIds: string[];
  maskAccountText: (value?: string | null) => string;
  onClose: () => void;
  onSaveAccounts: (payload: { accountIds: string[]; restrictFreeAccounts: boolean }) => Promise<unknown> | unknown;
  onClearStats: () => Promise<unknown> | unknown;
  onRefreshStats: () => Promise<unknown> | unknown;
  onUpdatePort: (port: number) => Promise<unknown> | unknown;
  onUpdateRoutingStrategy: (strategy: CodexLocalAccessRoutingStrategy) => Promise<unknown> | unknown;
  onRotateApiKey: () => Promise<unknown> | unknown;
  onKillPort: () => Promise<unknown> | unknown;
  onToggleEnabled: () => Promise<unknown> | unknown;
  onTest: () => Promise<number> | number;
  saving: boolean;
  testing: boolean;
  starting: boolean;
  portCleanupBusy: boolean;
}

type CopyableField = 'baseUrl' | 'apiKey' | 'modelId';

export function CodexLocalAccessModal({
  isOpen, mode, state, accounts, accountGroups, initialSelectedIds, maskAccountText,
  onClose, onSaveAccounts, onClearStats, onRefreshStats, onUpdatePort,
  onUpdateRoutingStrategy, onRotateApiKey, onKillPort, onToggleEnabled,
  onTest, saving, testing, starting, portCleanupBusy,
}: CodexLocalAccessModalProps) {
  const collection = state?.collection ?? null;
  const baseUrl = state?.baseUrl ?? '';
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [query, setQuery] = useState('');
  const [restrictFreeAccounts, setRestrictFreeAccounts] = useState(true);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [portInput, setPortInput] = useState('');
  const [keyVisible, setKeyVisible] = useState(false);
  const [copiedField, setCopiedField] = useState<CopyableField | null>(null);
  const [filterTypes, setFilterTypes] = useState<string[]>([]);
  const [tagFilter, setTagFilter] = useState<string[]>([]);
  const [groupFilter, setGroupFilter] = useState<string[]>([]);
  const [localTesting, setLocalTesting] = useState(false);
  const actionBusy = saving || testing || starting || portCleanupBusy;

  const oauthAccounts = useMemo(() => accounts.filter(a => !isCodexApiKeyAccount(a)), [accounts]);
  const oauthAccountIdSet = useMemo(() => new Set(oauthAccounts.map(a => a.id)), [oauthAccounts]);

  useEffect(() => {
    if (!isOpen) return;
    setSelected(new Set(initialSelectedIds.filter(id => oauthAccountIdSet.has(id))));
    setQuery('');
    setFilterTypes([]);
    setTagFilter([]);
    setGroupFilter([]);
    setRestrictFreeAccounts(collection?.restrictFreeAccounts ?? true);
    setError('');
    setNotice('');
    setKeyVisible(false);
    setPortInput(collection?.port ? String(collection.port) : '');
  }, [isOpen, initialSelectedIds, oauthAccountIdSet, collection]);

  const visibleAccounts = useMemo(() => {
    const q = query.trim().toLowerCase();
    return oauthAccounts.filter(a => {
      const label = `${a.email || ''} ${a.display_name || ''} ${a.account_name || ''}`.toLowerCase();
      if (q && !label.includes(q)) return false;
      if (tagFilter.length > 0 && !(a.tags || []).some(t => tagFilter.includes(t.trim()))) return false;
      if (groupFilter.length > 0 && !accountGroups.some(g => groupFilter.includes(g.id) && g.accountIds.includes(a.id))) return false;
      if (filterTypes.length > 0) {
        const pk = getCodexPlanFilterKey(a);
        if (!filterTypes.some(t => t === pk || (t === 'ERROR' && a.quota_error))) return false;
      }
      return true;
    });
  }, [oauthAccounts, query, tagFilter, groupFilter, filterTypes, accountGroups]);

  const handleCopy = async (field: CopyableField, value: string) => {
    try {
      await navigator.clipboard.writeText(value);
      setCopiedField(field);
      window.setTimeout(() => setCopiedField(c => c === field ? null : c), 1200);
    } catch {
      setError('Copy failed');
    }
  };

  const toggleSelect = (id: string) => {
    setSelected(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  };

  const handleSaveMembers = async () => {
    try {
      const selectedIds = Array.from(selected).filter((id) => {
        if (!restrictFreeAccounts) return true;
        const account = oauthAccounts.find(a => a.id === id);
        return account ? getCodexPlanFilterKey(account) !== 'FREE' : false;
      });
      await onSaveAccounts({ accountIds: selectedIds, restrictFreeAccounts });
      onClose();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleSavePort = async () => {
    const nextPort = Number(portInput.trim());
    if (!Number.isInteger(nextPort) || nextPort <= 0 || nextPort > 65535) {
      setError('Port must be 1-65535');
      return;
    }
    try {
      await onUpdatePort(nextPort);
      setNotice('Port updated');
    } catch (e) { setError(String(e)); }
  };

  const handleTest = async () => {
    setLocalTesting(true);
    setError('');
    try {
      const status = await onTest();
      setNotice(status === 0 ? 'Service test completed' : `Service test returned ${status}`);
      await onRefreshStats();
    } catch (e) {
      setError(String(e));
    } finally {
      setLocalTesting(false);
    }
  };

  const routingStrategyOptions = [
    { value: 'auto', label: 'Auto (recommended)' },
    { value: 'quota_high_first', label: 'Quota High First' },
    { value: 'quota_low_first', label: 'Quota Low First' },
    { value: 'plan_high_first', label: 'Plan High First' },
    { value: 'plan_low_first', label: 'Plan Low First' },
    { value: 'expiry_soon_first', label: 'Expiry Soon First' },
  ];
  const routingStrategy = collection?.routingStrategy ?? 'auto';

  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className={`modal ${mode === 'members' ? 'group-account-picker-modal' : ''}`} onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2><Server size={18} /> {mode === 'members' ? 'API Service Members' : 'Local API Service'}</h2>
          <button className="modal-close" onClick={onClose}><X /></button>
        </div>
        <div className="modal-body">
          {error && <div className="form-error"><CircleAlert size={14} /> {error}</div>}
          {notice && <div className="form-success"><Check size={14} /> {notice}</div>}

          {mode === 'panel' && (
            <>
              <div className="local-access-status-row">
                <span className={`status-badge ${state?.running ? 'running' : 'stopped'}`}>
                  {collection?.enabled ? (state?.running ? 'Running' : 'Stopped') : 'Disabled'}
                </span>
                <button className={`btn btn-sm ${collection?.enabled ? 'btn-danger' : 'btn-primary'}`}
                  onClick={onToggleEnabled} disabled={actionBusy}>
                  <Power size={14} /> {collection?.enabled ? 'Disable' : 'Enable'}
                </button>
              </div>

              {collection && (
                <div className="local-access-config">
                  <div className="config-row">
                    <span>Base URL</span>
                    <code>{baseUrl}</code>
                    <button className="icon-btn" onClick={() => handleCopy('baseUrl', baseUrl)}>
                      {copiedField === 'baseUrl' ? <Check size={14} /> : <Copy size={14} />}
                    </button>
                  </div>
                  <div className="config-row">
                    <span>API Key</span>
                    <code>{keyVisible ? collection.apiKey : `${collection.apiKey.slice(0, 10)}\u2022\u2022\u2022\u2022`}</code>
                    <button className="icon-btn" onClick={() => setKeyVisible(!keyVisible)}>
                      {keyVisible ? <EyeOff size={14} /> : <Eye size={14} />}
                    </button>
                    <button className="icon-btn" onClick={() => handleCopy('apiKey', collection.apiKey)}>
                      {copiedField === 'apiKey' ? <Check size={14} /> : <Copy size={14} />}
                    </button>
                  </div>
                  <div className="config-row">
                    <span>Port</span>
                    <input type="number" className="form-input port-input" value={portInput} onChange={e => setPortInput(e.target.value)} min={1} max={65535} disabled={actionBusy} />
                    <button className="btn btn-secondary btn-sm" onClick={handleSavePort} disabled={actionBusy}>
                      <Gauge size={14} /> Save
                    </button>
                  </div>
                  <div className="config-row">
                    <span>Routing</span>
                    <select className="form-select" value={routingStrategy}
                      onChange={e => onUpdateRoutingStrategy(e.target.value as CodexLocalAccessRoutingStrategy)}
                      disabled={actionBusy}>
                      {routingStrategyOptions.map(o => <option key={o.value} value={o.value}>{o.label}</option>)}
                    </select>
                  </div>
                </div>
              )}

              <div className="local-access-actions">
                <button className="btn btn-secondary btn-sm" onClick={handleTest} disabled={!collection || actionBusy || localTesting}>
                  <Wrench size={14} /> {localTesting ? 'Testing...' : 'Test'}
                </button>
                <button className="btn btn-secondary btn-sm" onClick={onRefreshStats} disabled={!collection || actionBusy}>
                  <RefreshCw size={14} /> Refresh Stats
                </button>
                <button className="btn btn-secondary btn-sm" onClick={onRotateApiKey} disabled={!collection || actionBusy}>
                  <RefreshCw size={14} /> Rotate Key
                </button>
                <button className="btn btn-secondary btn-sm" onClick={onClearStats} disabled={!collection || actionBusy}>
                  <Trash2 size={14} /> Clear Stats
                </button>
                <button className="btn btn-secondary btn-sm" onClick={onKillPort} disabled={actionBusy}>
                  <Wrench size={14} /> Kill Port
                </button>
              </div>
              {state?.stats && (
                <div className="local-access-stats-grid">
                  {[
                    { label: 'Daily', totals: state.stats.daily.totals },
                    { label: 'Weekly', totals: state.stats.weekly.totals },
                    { label: 'Monthly', totals: state.stats.monthly.totals },
                  ].map(({ label, totals }) => (
                    <div className="local-access-stat-card" key={label}>
                      <span>{label}</span>
                      <strong>{totals.requestCount}</strong>
                      <small>{totals.successCount} ok / {totals.failureCount} failed</small>
                      <small>{totals.totalTokens} tokens</small>
                    </div>
                  ))}
                </div>
              )}
            </>
          )}

          {mode === 'members' && (
            <>
              <div className="group-account-toolbar">
                <div className="search-box">
                  <Search size={16} className="search-icon" />
                  <input type="text" value={query} onChange={e => setQuery(e.target.value)} placeholder="Search accounts..." />
                </div>
                <label className="checkbox-label">
                  <input type="checkbox" checked={restrictFreeAccounts} onChange={() => setRestrictFreeAccounts(!restrictFreeAccounts)} />
                  Restrict FREE accounts
                </label>
              </div>
              <div className="group-account-list">
                {visibleAccounts.map(account => {
                  const presentation = buildCodexAccountPresentation(account);
                  const freeBlocked = restrictFreeAccounts && getCodexPlanFilterKey(account) === 'FREE';
                  return (
                    <label key={account.id} className={`group-account-item ${selected.has(account.id) ? 'selected' : ''} ${freeBlocked ? 'disabled' : ''}`}>
                      <input type="checkbox" checked={selected.has(account.id)} onChange={() => toggleSelect(account.id)} disabled={actionBusy || freeBlocked} />
                      <span className="group-account-email">{maskAccountText(account.account_name || account.display_name || account.email)}</span>
                      <span className={`plan-badge ${presentation.planClass}`}>
                        {presentation.planLabel}
                      </span>
                    </label>
                  );
                })}
              </div>
              {visibleAccounts.length === 0 && <div className="empty-state-sm">No accounts match</div>}
            </>
          )}
        </div>
        <div className="modal-footer">
          <button className="btn btn-secondary" onClick={onClose}>Cancel</button>
          {mode === 'members' && (
            <button className="btn btn-primary" onClick={handleSaveMembers} disabled={actionBusy || selected.size === 0}>
              Save ({selected.size})
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
