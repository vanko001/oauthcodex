import { useState, useEffect, useCallback } from 'react';
import { X, FolderOpen, Plus, Pencil, Trash2, FolderPlus, AlertCircle } from 'lucide-react';
import { useCodexUiStore } from '../stores/useCodexUiStore';

interface CodexAccountGroupModalProps {
  isOpen: boolean;
  onClose: () => void;
  onGroupsChanged: () => Promise<void> | void;
  groupFilter?: string[];
  onToggleGroupFilter?: (groupId: string) => void;
  onClearGroupFilter?: () => void;
}

export function CodexAccountGroupModal({
  isOpen, onClose, onGroupsChanged,
  groupFilter = [], onToggleGroupFilter, onClearGroupFilter,
}: CodexAccountGroupModalProps) {
  const groups = useCodexUiStore(s => s.accountGroups);
  const fetchAccountGroups = useCodexUiStore(s => s.fetchAccountGroups);
  const createAccountGroup = useCodexUiStore(s => s.createAccountGroup);
  const renameAccountGroup = useCodexUiStore(s => s.renameAccountGroup);
  const deleteAccountGroup = useCodexUiStore(s => s.deleteAccountGroup);
  const [newName, setNewName] = useState('');
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    await fetchAccountGroups();
  }, [fetchAccountGroups]);

  useEffect(() => {
    if (isOpen) { reload(); setNewName(''); setRenamingId(null); setDeleteConfirmId(null); setError(null); }
  }, [isOpen, reload]);

  const handleCreate = async () => {
    const name = newName.trim();
    if (!name) return;
    try {
      if (groups.some(g => g.name === name)) { setError('Group name already exists'); return; }
      await createAccountGroup(name);
      setNewName('');
      await reload();
      await onGroupsChanged();
    } catch (e) { setError(String(e)); }
  };

  const handleRename = async (id: string) => {
    const name = renameValue.trim();
    if (!name) return;
    try {
      await renameAccountGroup(id, name);
      setRenamingId(null);
      await reload();
      await onGroupsChanged();
    } catch (e) { setError(String(e)); }
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteAccountGroup(id);
      setDeleteConfirmId(null);
      await reload();
      await onGroupsChanged();
    } catch (e) { setError(String(e)); }
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2><FolderOpen size={18} /> Group Management</h2>
          <button className="modal-close" onClick={onClose}><X /></button>
        </div>
        <div className="modal-body">
          <div className="group-create-row">
            <input type="text" value={newName} onChange={e => setNewName(e.target.value)}
              onKeyDown={e => { if (e.key === 'Enter') handleCreate(); }}
              placeholder="New group name..." maxLength={30} />
            <button className="btn btn-primary" onClick={handleCreate} disabled={!newName.trim()}>
              <Plus size={14} /> Create
            </button>
          </div>
          {error && <div className="form-error"><AlertCircle size={14} /> {error}</div>}
          {groupFilter.length > 0 && onClearGroupFilter && (
            <div className="filter-hint">
              <span>{groupFilter.length} group(s) selected for filter</span>
              <button className="btn btn-secondary btn-sm" onClick={onClearGroupFilter}>Clear filter</button>
            </div>
          )}
          <div className="group-list">
            {groups.map(group => (
              <div key={group.id} className={`group-item ${groupFilter.includes(group.id) ? 'filtered' : ''}`}>
                {onToggleGroupFilter && (
                  <input type="checkbox" checked={groupFilter.includes(group.id)}
                    onChange={() => onToggleGroupFilter(group.id)} />
                )}
                <FolderOpen size={16} className="group-icon" />
                {renamingId === group.id ? (
                  <input className="inline-rename-input" value={renameValue} onChange={e => setRenameValue(e.target.value)}
                    onKeyDown={e => { if (e.key === 'Enter') handleRename(group.id); if (e.key === 'Escape') setRenamingId(null); }}
                    onBlur={() => handleRename(group.id)} autoFocus maxLength={30} />
                ) : (
                  <>
                    <span className="group-name">{group.name}</span>
                    <span className="group-count">{group.accountIds.length} accounts</span>
                  </>
                )}
                <div className="group-actions">
                  {deleteConfirmId === group.id ? (
                    <>
                      <button className="icon-btn danger" onClick={() => handleDelete(group.id)}>✓</button>
                      <button className="icon-btn" onClick={() => setDeleteConfirmId(null)}>✗</button>
                    </>
                  ) : (
                    <>
                      <button className="icon-btn" onClick={() => { setRenamingId(group.id); setRenameValue(group.name); }}>
                        <Pencil size={14} />
                      </button>
                      <button className="icon-btn danger" onClick={() => setDeleteConfirmId(group.id)}>
                        <Trash2 size={14} />
                      </button>
                    </>
                  )}
                </div>
              </div>
            ))}
          </div>
          {groups.length === 0 && <div className="empty-state-sm"><FolderPlus size={24} /><p>No groups yet</p></div>}
        </div>
      </div>
    </div>
  );
}

interface CodexAddToGroupModalProps {
  isOpen: boolean;
  onClose: () => void;
  accountIds: string[];
  sourceGroupId?: string;
  onAdded: () => Promise<void> | void;
}

export function CodexAddToGroupModal({ isOpen, onClose, accountIds, sourceGroupId, onAdded }: CodexAddToGroupModalProps) {
  const groups = useCodexUiStore(s => s.accountGroups);
  const fetchAccountGroups = useCodexUiStore(s => s.fetchAccountGroups);
  const assignToGroup = useCodexUiStore(s => s.assignToGroup);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen) { void fetchAccountGroups(); setError(null); }
  }, [isOpen, fetchAccountGroups]);

  const handleSelect = async (groupId: string) => {
    try {
      await assignToGroup(groupId, accountIds);
      await onAdded();
      onClose();
    } catch (e) { setError(String(e)); }
  };

  if (!isOpen) return null;
  const selectable = groups.filter(g => g.id !== sourceGroupId);

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal modal-sm" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2><FolderPlus size={18} /> Add to Group</h2>
          <button className="modal-close" onClick={onClose}><X /></button>
        </div>
        <div className="modal-body">
          {selectable.map(group => (
            <div key={group.id} className="add-to-group-item" onClick={() => handleSelect(group.id)}>
              <FolderOpen size={16} />
              <span className="group-name">{group.name}</span>
              <span className="group-count">{group.accountIds.length}</span>
            </div>
          ))}
          {error && <div className="form-error">{error}</div>}
          {selectable.length === 0 && <p>No other groups available</p>}
        </div>
      </div>
    </div>
  );
}
