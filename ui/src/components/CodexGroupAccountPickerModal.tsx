import { useState, useEffect, useMemo } from 'react';
import { FolderPlus, Search, X } from 'lucide-react';
import type { CodexAccount } from '../types/codex';
import type { CodexAccountGroup } from '../stores/useCodexUiStore';
import { buildCodexAccountPresentation } from '../presentation/codexPresentation';

interface CodexGroupAccountPickerModalProps {
  isOpen: boolean;
  targetGroup: CodexAccountGroup | null | undefined;
  accounts: CodexAccount[];
  accountGroups: CodexAccountGroup[];
  maskAccountText: (value?: string | null) => string;
  onClose: () => void;
  onConfirm: (payload: { accountIds: string[] }) => Promise<void> | void;
}

export function CodexGroupAccountPickerModal({
  isOpen, targetGroup, accounts, maskAccountText, onClose, onConfirm,
}: CodexGroupAccountPickerModalProps) {
  const [query, setQuery] = useState('');
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  useEffect(() => {
    if (!isOpen) return;
    setQuery(''); setSelected(new Set()); setError('');
  }, [isOpen, targetGroup]);

  const existingIds = useMemo(() => new Set(targetGroup?.accountIds ?? []), [targetGroup]);
  const visibleAccounts = useMemo(() => {
    const q = query.trim().toLowerCase();
    return accounts.filter(a => !existingIds.has(a.id) && (!q || a.email.toLowerCase().includes(q)));
  }, [accounts, existingIds, query]);

  const handleConfirm = async () => {
    if (!targetGroup || selected.size === 0 || saving) return;
    setSaving(true);
    try {
      await onConfirm({ accountIds: Array.from(selected) });
      onClose();
    } catch (e) { setError(String(e)); }
    finally { setSaving(false); }
  };

  if (!isOpen || !targetGroup) return null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal group-account-picker-modal" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2><FolderPlus size={18} /> Add Accounts to {targetGroup.name}</h2>
          <button className="modal-close" onClick={onClose}><X /></button>
        </div>
        <div className="modal-body">
          <div className="search-box">
            <Search size={16} className="search-icon" />
            <input type="text" value={query} onChange={e => setQuery(e.target.value)} placeholder="Search accounts..." />
          </div>
          <div className="group-account-list">
            {visibleAccounts.map(account => {
              const pres = buildCodexAccountPresentation(account);
              return (
                <label key={account.id} className={`group-account-item ${selected.has(account.id) ? 'selected' : ''}`}>
                  <input type="checkbox" checked={selected.has(account.id)} onChange={() => setSelected(prev => {
                    const next = new Set(prev);
                    if (next.has(account.id)) next.delete(account.id); else next.add(account.id);
                    return next;
                  })} disabled={saving} />
                  <span className="group-account-email">{maskAccountText(account.email)}</span>
                  <span className={`plan-badge ${pres.planClass}`}>{pres.planLabel}</span>
                </label>
              );
            })}
          </div>
          {error && <div className="form-error">{error}</div>}
        </div>
        <div className="modal-footer">
          <button className="btn btn-secondary" onClick={onClose} disabled={saving}>Cancel</button>
          <button className="btn btn-primary" onClick={handleConfirm} disabled={selected.size === 0 || saving}>
            Add ({selected.size})
          </button>
        </div>
      </div>
    </div>
  );
}
