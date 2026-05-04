import { useState, useEffect, useMemo } from 'react';
import { Play, X, RefreshCw, Trash2, Power, Search, Plus } from 'lucide-react';
import { useCodexUiStore } from '../stores/useCodexUiStore';
import type { CodexWakeupTask } from '../types/codexWakeup';
import { isCodexApiKeyAccount } from '../types/codex';

export function CodexWakeupPage() {
  const store = useCodexUiStore();
  const [notice, setNotice] = useState<{ text: string; tone?: string } | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [taskName, setTaskName] = useState('');
  const [taskPrompt, setTaskPrompt] = useState('');
  const [taskTime, setTaskTime] = useState('09:00');
  const [taskModelPresetId, setTaskModelPresetId] = useState('');
  const [taskAccountIds, setTaskAccountIds] = useState<Set<string>>(new Set());

  useEffect(() => {
    store.fetchWakeup();
  }, []);

  const { wakeupRuntime, wakeupState, wakeupHistory } = store;
  const tasks = useMemo(() => {
    let list = [...wakeupState.tasks];
    const query = searchQuery.trim().toLowerCase();
    if (query) list = list.filter(t => t.name.toLowerCase().includes(query));
    list.sort((a, b) => {
      if (a.enabled !== b.enabled) return a.enabled ? -1 : 1;
      return (b.next_run_at || 0) - (a.next_run_at || 0);
    });
    return list;
  }, [wakeupState.tasks, searchQuery]);

  const handleRunTask = async (taskId: string) => {
    try {
      await store.runWakeupTask(taskId);
      setNotice({ text: 'Task running...', tone: 'success' });
    } catch (e) {
      setNotice({ text: String(e), tone: 'error' });
    }
  };

  const handleToggleEnabled = async (task: CodexWakeupTask) => {
    const updatedTasks = wakeupState.tasks.map(t => t.id === task.id ? { ...t, enabled: !t.enabled } : t);
    await store.saveWakeupState(wakeupState.enabled, updatedTasks, wakeupState.model_presets);
  };

  const handleToggleGlobalEnabled = async () => {
    await store.saveWakeupState(!wakeupState.enabled, wakeupState.tasks, wakeupState.model_presets);
  };

  const openCreateModal = () => {
    setTaskName('');
    setTaskPrompt('');
    setTaskTime('09:00');
    setTaskModelPresetId(wakeupState.model_presets[0]?.id || '');
    setTaskAccountIds(new Set(store.accounts.filter(a => !isCodexApiKeyAccount(a)).map(a => a.id)));
    setShowCreateModal(true);
  };

  const handleCreateTask = async () => {
    const name = taskName.trim();
    const prompt = taskPrompt.trim();
    if (!name || !prompt || taskAccountIds.size === 0) return;
    const preset = wakeupState.model_presets.find(p => p.id === taskModelPresetId);
    const now = Math.floor(Date.now() / 1000);
    const task: CodexWakeupTask = {
      id: `cwakeup_${Date.now()}`,
      name,
      enabled: true,
      account_ids: Array.from(taskAccountIds),
      prompt,
      model: preset?.model,
      model_display_name: preset?.name,
      model_reasoning_effort: preset?.default_reasoning_effort,
      schedule: {
        kind: 'daily',
        daily_time: taskTime,
        weekly_days: [],
      },
      created_at: now,
      updated_at: now,
    };
    try {
      await store.saveWakeupState(wakeupState.enabled, [...wakeupState.tasks, task], wakeupState.model_presets);
      setShowCreateModal(false);
      setNotice({ text: 'Task created', tone: 'success' });
    } catch (e) {
      setNotice({ text: String(e), tone: 'error' });
    }
  };

  const handleDeleteTask = async (taskId: string) => {
    if (!window.confirm('Delete this wakeup task?')) return;
    const updatedTasks = wakeupState.tasks.filter(t => t.id !== taskId);
    await store.saveWakeupState(wakeupState.enabled, updatedTasks, wakeupState.model_presets);
  };

  const toggleTaskAccount = (accountId: string) => {
    setTaskAccountIds(prev => {
      const next = new Set(prev);
      if (next.has(accountId)) next.delete(accountId); else next.add(accountId);
      return next;
    });
  };

  return (
    <div className="codex-wakeup-page">
      <div className="page-header">
        <h1>Codex Wakeup</h1>
        <div className="page-header-stats">
          <button className={`btn btn-sm ${wakeupState.enabled ? 'btn-danger' : 'btn-primary'}`} onClick={handleToggleGlobalEnabled}>
            <Power size={14} /> {wakeupState.enabled ? 'Disable' : 'Enable'}
          </button>
          <span className={`status-badge ${wakeupRuntime?.available ? 'running' : 'stopped'}`}>
            CLI: {wakeupRuntime?.available ? 'Available' : 'Unavailable'}
          </span>
          <span>{wakeupState.tasks.length} tasks</span>
        </div>
      </div>

      {notice && (
        <div className={`message-bar ${notice.tone === 'error' ? 'error' : 'success'}`}>
          {notice.text}
          <button className="message-close" onClick={() => setNotice(null)}><X size={14} /></button>
        </div>
      )}

      <div className="page-toolbar">
        <div className="toolbar-left">
          <div className="search-box">
            <Search size={16} className="search-icon" />
            <input type="text" placeholder="Search tasks..." value={searchQuery} onChange={e => setSearchQuery(e.target.value)} />
          </div>
        </div>
        <div className="toolbar-right">
          <button className="btn btn-primary btn-sm" onClick={openCreateModal}>
            <Plus size={14} /> Add Task
          </button>
          <button className="btn btn-secondary btn-sm" onClick={store.fetchWakeup}>
            <RefreshCw size={14} /> Refresh
          </button>
        </div>
      </div>

      <div className="runtime-card">
        <div className="runtime-card-header">
          <Power size={16} />
          <span>Runtime Status</span>
          <span className={`status-badge ${wakeupRuntime?.available ? 'running' : 'stopped'}`}>
            {wakeupRuntime?.available ? 'Available' : 'Unavailable'}
          </span>
        </div>
        {wakeupRuntime && (
          <div className="runtime-details">
            {wakeupRuntime.binary_path && <div className="runtime-detail"><span>Binary:</span><code>{wakeupRuntime.binary_path}</code></div>}
            {wakeupRuntime.configured_codex_cli_path && <div className="runtime-detail"><span>CLI Path:</span><code>{wakeupRuntime.configured_codex_cli_path}</code></div>}
            {wakeupRuntime.version && <div className="runtime-detail"><span>Version:</span><span>{wakeupRuntime.version}</span></div>}
          </div>
        )}
      </div>

      {tasks.length === 0 ? (
        <div className="empty-state">
          <h3>No wakeup tasks</h3>
          <p>Create tasks to automatically run prompts on schedule.</p>
        </div>
      ) : (
        <div className="wakeup-task-list">
          {tasks.map(task => (
            <div key={task.id} className={`wakeup-task-card ${task.enabled ? 'enabled' : 'disabled'}`}>
              <div className="wakeup-task-header">
                <input type="checkbox" checked={task.enabled} onChange={() => handleToggleEnabled(task)} />
                <span className="wakeup-task-name">{task.name}</span>
                <span className={`task-status ${task.last_status || 'idle'}`}>
                  {task.last_status || 'Idle'}
                </span>
              </div>
              <div className="wakeup-task-body">
                <div className="wakeup-task-detail">
                  <span>Schedule:</span>
                  <span>{task.schedule.kind} {task.schedule.daily_time || task.schedule.weekly_time || ''}</span>
                </div>
                <div className="wakeup-task-detail">
                  <span>Accounts:</span>
                  <span>{task.account_ids.length}</span>
                </div>
                {task.model && (
                  <div className="wakeup-task-detail">
                    <span>Model:</span>
                    <span>{task.model_display_name || task.model}</span>
                  </div>
                )}
                {task.prompt && (
                  <div className="wakeup-task-detail">
                    <span>Prompt:</span>
                    <code className="wakeup-prompt-preview">{task.prompt.slice(0, 100)}</code>
                  </div>
                )}
              </div>
              <div className="wakeup-task-actions">
                <button className="icon-btn" onClick={() => handleRunTask(task.id)} title="Run now">
                  <Play size={14} />
                </button>
                <button className="icon-btn danger" onClick={() => handleDeleteTask(task.id)} title="Delete task">
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {wakeupHistory.length > 0 && (
        <div className="wakeup-history-section">
          <div className="section-header">
            <h3>History ({wakeupHistory.length})</h3>
            <button className="btn btn-danger btn-sm" onClick={store.clearWakeupHistory}>
              <Trash2 size={14} /> Clear
            </button>
          </div>
          <div className="wakeup-history-list">
            {wakeupHistory.slice(0, 50).map(item => (
              <div key={item.id} className={`history-item ${item.success ? 'success' : 'error'}`}>
                <span className="history-email">{item.account_email}</span>
                <span className={`history-status ${item.success ? 'success' : 'error'}`}>
                  {item.success ? 'OK' : 'FAIL'}
                </span>
                <span className="history-time">{new Date(item.timestamp * 1000).toLocaleString()}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {showCreateModal && (
        <div className="modal-overlay" onClick={() => setShowCreateModal(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Create Wakeup Task</h2>
              <button className="modal-close" onClick={() => setShowCreateModal(false)}><X /></button>
            </div>
            <div className="modal-body">
              <div className="form-group">
                <label>Name</label>
                <input className="form-input" value={taskName} onChange={e => setTaskName(e.target.value)} />
              </div>
              <div className="form-group">
                <label>Daily Time</label>
                <input type="time" className="form-input" value={taskTime} onChange={e => setTaskTime(e.target.value)} />
              </div>
              <div className="form-group">
                <label>Prompt</label>
                <textarea className="form-textarea" rows={4} value={taskPrompt} onChange={e => setTaskPrompt(e.target.value)} />
              </div>
              {wakeupState.model_presets.length > 0 && (
                <div className="form-group">
                  <label>Model Preset</label>
                  <select className="form-select" value={taskModelPresetId} onChange={e => setTaskModelPresetId(e.target.value)}>
                    {wakeupState.model_presets.map(preset => (
                      <option key={preset.id} value={preset.id}>{preset.name}</option>
                    ))}
                  </select>
                </div>
              )}
              <div className="form-group">
                <label>Accounts</label>
                <div className="wakeup-account-list">
                  {store.accounts.filter(a => !isCodexApiKeyAccount(a)).map(account => (
                    <label key={account.id} className="wakeup-account-row">
                      <input type="checkbox" checked={taskAccountIds.has(account.id)} onChange={() => toggleTaskAccount(account.id)} />
                      <span>{account.account_name || account.display_name || account.email}</span>
                    </label>
                  ))}
                </div>
              </div>
            </div>
            <div className="modal-footer">
              <button className="btn btn-secondary" onClick={() => setShowCreateModal(false)}>Cancel</button>
              <button className="btn btn-primary" onClick={handleCreateTask} disabled={!taskName.trim() || !taskPrompt.trim() || taskAccountIds.size === 0}>
                Create
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
