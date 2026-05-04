import { useEffect, useMemo, useState } from 'react';
import { Folder, RefreshCw, RotateCcw, Trash2, Wrench } from 'lucide-react';
import { codexClient } from '../services/codexClient';

type UiSessionRecord = {
  sessionId?: string;
  session_id?: string;
  id?: string;
  title?: string;
  cwd?: string;
  updatedAt?: number | null;
  updated_at?: number | null;
  locationCount?: number;
  location_count?: number;
};

function getSessionId(session: UiSessionRecord): string {
  return session.sessionId || session.session_id || session.id || '';
}

function getUpdatedAt(session: UiSessionRecord): number | null {
  return session.updatedAt ?? session.updated_at ?? null;
}

export function CodexSessionManager() {
  const [sessions, setSessions] = useState<UiSessionRecord[]>([]);
  const [trashedSessions, setTrashedSessions] = useState<UiSessionRecord[]>([]);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [showTrash, setShowTrash] = useState(false);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<{ text: string; tone?: string } | null>(null);

  const activeList = showTrash ? trashedSessions : sessions;
  const selectedActiveIds = useMemo(
    () => activeList.map(getSessionId).filter(id => selectedIds.has(id)),
    [activeList, selectedIds],
  );

  const refresh = async () => {
    setBusy(true);
    try {
      const [nextSessions, nextTrash] = await Promise.all([
        codexClient.listSessionsAcrossInstances(),
        codexClient.listTrashedSessionsAcrossInstances(),
      ]);
      setSessions(nextSessions as UiSessionRecord[]);
      setTrashedSessions(nextTrash as UiSessionRecord[]);
      setSelectedIds(new Set());
    } catch (e) {
      setMessage({ text: String(e), tone: 'error' });
    } finally {
      setBusy(false);
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  const toggleSelected = (id: string) => {
    setSelectedIds(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  };

  const handleSync = async () => {
    setBusy(true);
    try {
      const result = await codexClient.syncThreadsAcrossInstances();
      setMessage({ text: result.message, tone: 'success' });
      await refresh();
    } catch (e) {
      setMessage({ text: String(e), tone: 'error' });
    } finally {
      setBusy(false);
    }
  };

  const handleRepair = async () => {
    setBusy(true);
    try {
      const result = await codexClient.repairSessionVisibilityAcrossInstances();
      setMessage({ text: result.message, tone: 'success' });
      await refresh();
    } catch (e) {
      setMessage({ text: String(e), tone: 'error' });
    } finally {
      setBusy(false);
    }
  };

  const handleTrash = async () => {
    if (selectedActiveIds.length === 0) return;
    setBusy(true);
    try {
      const result = await codexClient.moveSessionsToTrashAcrossInstances(selectedActiveIds);
      setMessage({ text: result.message, tone: 'success' });
      await refresh();
    } catch (e) {
      setMessage({ text: String(e), tone: 'error' });
    } finally {
      setBusy(false);
    }
  };

  const handleRestore = async () => {
    if (selectedActiveIds.length === 0) return;
    setBusy(true);
    try {
      const result = await codexClient.restoreSessionsFromTrashAcrossInstances(selectedActiveIds);
      setMessage({ text: result.message, tone: 'success' });
      await refresh();
    } catch (e) {
      setMessage({ text: String(e), tone: 'error' });
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="codex-session-manager">
      <div className="session-manager-header">
        <button className="btn btn-secondary btn-sm" onClick={refresh} disabled={busy}>
          <RefreshCw size={14} /> Refresh
        </button>
        <button className="btn btn-secondary btn-sm" onClick={handleSync} disabled={busy}>
          <RotateCcw size={14} /> Sync Threads
        </button>
        <button className="btn btn-secondary btn-sm" onClick={handleRepair} disabled={busy}>
          <Wrench size={14} /> Repair Visibility
        </button>
        <button className={`btn btn-secondary btn-sm ${showTrash ? 'active' : ''}`} onClick={() => { setShowTrash(v => !v); setSelectedIds(new Set()); }}>
          <Trash2 size={14} /> Trash ({trashedSessions.length})
        </button>
      </div>
      {message && <div className={`message-bar ${message.tone === 'error' ? 'error' : 'success'}`}>{message.text}</div>}

      {activeList.length === 0 ? (
        <div className="empty-state">
          <Folder size={42} />
          <h3>{showTrash ? 'No trashed sessions' : 'No sessions found'}</h3>
        </div>
      ) : (
        <>
          <div className="session-bulk-actions">
            <span>{selectedActiveIds.length} selected</span>
            {!showTrash ? (
              <button className="btn btn-danger btn-sm" onClick={handleTrash} disabled={busy || selectedActiveIds.length === 0}>
                <Trash2 size={14} /> Move to Trash
              </button>
            ) : (
              <button className="btn btn-primary btn-sm" onClick={handleRestore} disabled={busy || selectedActiveIds.length === 0}>
                <RotateCcw size={14} /> Restore
              </button>
            )}
          </div>
          <div className="session-list">
            {activeList.map(session => {
              const sessionId = getSessionId(session);
              const updatedAt = getUpdatedAt(session);
              return (
                <label key={sessionId} className={`session-row ${selectedIds.has(sessionId) ? 'selected' : ''}`}>
                  <input type="checkbox" checked={selectedIds.has(sessionId)} onChange={() => toggleSelected(sessionId)} />
                  <div className="session-row-main">
                    <span className="session-title">{session.title || sessionId}</span>
                    {session.cwd && <code className="session-cwd">{session.cwd}</code>}
                  </div>
                  <span className="session-location-count">{session.locationCount ?? session.location_count ?? 0} locations</span>
                  {updatedAt && <span className="session-updated">{new Date(updatedAt * 1000).toLocaleString()}</span>}
                </label>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}
