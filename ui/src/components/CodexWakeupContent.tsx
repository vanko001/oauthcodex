import { useMemo } from 'react';
import { Play, Power } from 'lucide-react';
import { useCodexUiStore } from '../stores/useCodexUiStore';

interface CodexWakeupContentProps {
  accounts: unknown[];
  onRefreshAccounts: () => Promise<void>;
  openPresetManagerSignal?: number;
}

export function CodexWakeupContent(_props: CodexWakeupContentProps) {
  const store = useCodexUiStore();
  const { wakeupRuntime, wakeupState } = store;
  const sortedTasks = useMemo(() => {
    return [...wakeupState.tasks].sort((a, b) => {
      if (a.enabled !== b.enabled) return a.enabled ? -1 : 1;
      return 0;
    });
  }, [wakeupState.tasks]);

  return (
    <div className="codex-wakeup-content">
      <div className="runtime-indicator">
        <Power size={14} />
        <span>CLI: {wakeupRuntime?.available ? 'Available' : 'Unavailable'}</span>
      </div>
      {sortedTasks.length === 0 ? (
        <div className="empty-state">
          <h3>No wakeup tasks</h3>
          <p>Go to Wakeup page to create and manage tasks.</p>
        </div>
      ) : (
        <div className="wakeup-task-list">
          {sortedTasks.map(task => (
            <div key={task.id} className={`wakeup-task-row ${task.enabled ? 'enabled' : 'disabled'}`}>
              <span>{task.name}</span>
              <span>{task.schedule.kind}</span>
              <button className="icon-btn" onClick={() => store.runWakeupTask(task.id)}><Play size={14} /></button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
