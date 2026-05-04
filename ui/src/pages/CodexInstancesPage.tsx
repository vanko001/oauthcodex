import { useState, useEffect } from 'react';
import { ExternalLink, Plus, Play, Square, Terminal, X } from 'lucide-react';
import { useCodexUiStore } from '../stores/useCodexUiStore';
import type { InstanceLaunchMode, InstanceProfile } from '../types/instance';
import { isCodexApiKeyAccount } from '../types/codex';
import { codexClient } from '../services/codexClient';

export function CodexInstancesPage() {
  const store = useCodexUiStore();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [createName, setCreateName] = useState('');
  const [createUserDataDir, setCreateUserDataDir] = useState('');
  const [bindAccountId, setBindAccountId] = useState<string>('');
  const [launchMode, setLaunchMode] = useState<InstanceLaunchMode>('auto');
  const [message, setMessage] = useState<{ text: string; tone?: string } | null>(null);

  useEffect(() => {
    void store.fetchInstances();
  }, []);

  const openCreateModal = async () => {
    setShowCreateModal(true);
    if (!createUserDataDir) {
      try {
        const defaults = await codexClient.getInstanceDefaults();
        setCreateUserDataDir(defaults.defaultUserDataDir);
      } catch {
        setCreateUserDataDir('');
      }
    }
  };

  const handleCreate = async () => {
    if (!createName.trim() || !createUserDataDir.trim()) return;
    try {
      await store.createInstance({
        name: createName.trim(),
        userDataDir: createUserDataDir.trim(),
        copySourceInstanceId: '',
        bindAccountId: bindAccountId || null,
        launchMode,
      });
      setShowCreateModal(false);
      setCreateName('');
      setCreateUserDataDir('');
      setMessage({ text: 'Instance created', tone: 'success' });
    } catch (e) {
      setMessage({ text: String(e), tone: 'error' });
    }
  };

  const handleStartStop = async (instance: InstanceProfile) => {
    try {
      if (instance.running) {
        await store.stopInstance(instance.id);
      } else {
        await store.startInstance(instance.id);
      }
    } catch (e) {
      setMessage({ text: String(e), tone: 'error' });
    }
  };

  const handleDelete = async (id: string) => {
    if (!window.confirm('Delete this Codex instance?')) return;
    try {
      await store.deleteInstance(id);
    } catch (e) {
      setMessage({ text: String(e), tone: 'error' });
    }
  };

  const handleOpenWindow = async (id: string) => {
    try {
      await codexClient.openInstanceWindow(id);
    } catch (e) {
      setMessage({ text: String(e), tone: 'error' });
    }
  };

  const handleShowLaunchCommand = async (id: string) => {
    try {
      const result = await codexClient.getInstanceLaunchCommand(id);
      setMessage({ text: result.launchCommand, tone: 'success' });
    } catch (e) {
      setMessage({ text: String(e), tone: 'error' });
    }
  };

  return (
    <div className="codex-instances-page">
      <div className="page-header">
        <h1>Codex Instances</h1>
        <button className="btn btn-primary btn-sm" onClick={openCreateModal}>
          <Plus size={14} /> Create Instance
        </button>
      </div>

      {message && (
        <div className={`message-bar ${message.tone === 'error' ? 'error' : 'success'}`}>
          {message.text}
          <button className="message-close" onClick={() => setMessage(null)}><X size={14} /></button>
        </div>
      )}

      {store.instances.length === 0 ? (
        <div className="empty-state">
          <h3>No instances yet</h3>
          <p>Create your first Codex instance to get started.</p>
        </div>
      ) : (
        <div className="instances-grid">
          {store.instances.map(instance => (
            <div key={instance.id} className={`instance-card ${instance.running ? 'running' : ''}`}>
              <div className="instance-card-header">
                <span className="instance-name">{instance.name || 'Default'}</span>
                <span className={`instance-status ${instance.running ? 'running' : 'stopped'}`}>
                  {instance.running ? 'Running' : 'Stopped'}
                </span>
              </div>
              <div className="instance-card-body">
                <div className="instance-detail">
                  <span className="detail-label">Launch Mode</span>
                  <span className="detail-value">{instance.launchMode || 'auto'}</span>
                </div>
                {instance.bindAccountId && (
                  <div className="instance-detail">
                    <span className="detail-label">Bound Account</span>
                    <span className="detail-value">{instance.bindAccountId}</span>
                  </div>
                )}
                {instance.userDataDir && (
                  <div className="instance-detail">
                    <span className="detail-label">Data Dir</span>
                    <code className="detail-code">{instance.userDataDir}</code>
                  </div>
                )}
              </div>
              <div className="instance-card-actions">
                <button className="icon-btn" onClick={() => handleStartStop(instance)}
                  title={instance.running ? 'Stop' : 'Start'}>
                  {instance.running ? <Square size={14} /> : <Play size={14} />}
                </button>
                <button className="icon-btn" onClick={() => handleOpenWindow(instance.id)} title="Open window">
                  <ExternalLink size={14} />
                </button>
                <button className="icon-btn" onClick={() => handleShowLaunchCommand(instance.id)} title="Show launch command">
                  <Terminal size={14} />
                </button>
                <button className="icon-btn" onClick={() => handleDelete(instance.id)} title="Delete">
                  <X size={14} />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {showCreateModal && (
        <div className="modal-overlay" onClick={() => setShowCreateModal(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Create Instance</h2>
              <button className="modal-close" onClick={() => setShowCreateModal(false)}><X /></button>
            </div>
            <div className="modal-body">
              <div className="form-group">
                <label>Instance Name</label>
                <input className="form-input" value={createName} onChange={e => setCreateName(e.target.value)} placeholder="My Instance" />
              </div>
              <div className="form-group">
                <label>User Data Directory</label>
                <input className="form-input" value={createUserDataDir} onChange={e => setCreateUserDataDir(e.target.value)} placeholder="~/.codex/instances/my-instance" />
              </div>
              <div className="form-group">
                <label>Launch Mode</label>
                <select className="form-select" value={launchMode} onChange={e => setLaunchMode(e.target.value as any)}>
                  <option value="auto">Auto</option>
                  <option value="manual">Manual</option>
                  <option value="cli">CLI</option>
                </select>
              </div>
              <div className="form-group">
                <label>Bind Account (optional)</label>
                <select className="form-select" value={bindAccountId} onChange={e => setBindAccountId(e.target.value)}>
                  <option value="">None</option>
                  {store.accounts.filter(a => !isCodexApiKeyAccount(a)).map(a => (
                    <option key={a.id} value={a.id}>{a.email}</option>
                  ))}
                </select>
              </div>
              <button className="btn btn-primary" onClick={handleCreate} disabled={!createName.trim() || !createUserDataDir.trim()}>
                Create Instance
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
