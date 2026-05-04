import { useState, useEffect } from 'react';
import { AlarmClock, Boxes, Settings, Users } from 'lucide-react';
import { useCodexUiStore } from './stores/useCodexUiStore';
import { CodexAccountsPage } from './pages/CodexAccountsPage';
import { CodexInstancesPage } from './pages/CodexInstancesPage';
import { CodexWakeupPage } from './pages/CodexWakeupPage';
import { CodexSettingsPage } from './pages/CodexSettingsPage';
import type { CodexPage } from './types/navigation';

function App() {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const page = useCodexUiStore((s) => s.currentPage);
  const setPage = useCodexUiStore((s) => s.setCurrentPage);
  const fetchAccounts = useCodexUiStore((s) => s.fetchAccounts);
  const fetchCurrentAccount = useCodexUiStore((s) => s.fetchCurrentAccount);
  const fetchAccountGroups = useCodexUiStore((s) => s.fetchAccountGroups);
  const fetchLocalAccessState = useCodexUiStore((s) => s.fetchLocalAccessState);
  const currentAccount = useCodexUiStore((s) => s.currentAccount);

  useEffect(() => {
    void fetchAccounts();
    void fetchCurrentAccount();
    void fetchAccountGroups();
    void fetchLocalAccessState();
  }, [fetchAccounts, fetchCurrentAccount, fetchAccountGroups, fetchLocalAccessState]);

  const pages: { key: CodexPage; label: string }[] = [
    { key: 'accounts', label: 'Accounts' },
    { key: 'instances', label: 'Instances' },
    { key: 'wakeup', label: 'Wakeup' },
    { key: 'settings', label: 'Settings' },
  ];

  const renderPage = () => {
    switch (page) {
      case 'accounts': return <CodexAccountsPage />;
      case 'instances': return <CodexInstancesPage />;
      case 'wakeup': return <CodexWakeupPage />;
      case 'settings': return <CodexSettingsPage />;
    }
  };

  const renderNavIcon = (key: CodexPage) => {
    if (key === 'accounts') return <Users size={18} />;
    if (key === 'instances') return <Boxes size={18} />;
    if (key === 'wakeup') return <AlarmClock size={18} />;
    return <Settings size={18} />;
  };

  return (
    <div className="app-container">
      <nav className={`side-nav ${sidebarCollapsed ? 'collapsed' : ''}`}>
        <div className="side-nav-header" onClick={() => setSidebarCollapsed(!sidebarCollapsed)}>
          <div className="side-nav-brand">
            <span className="codex-icon">CX</span>
            {!sidebarCollapsed && <span className="brand-text">OAuth Codex</span>}
          </div>
        </div>
        <div className="side-nav-items">
          {pages.map((p) => (
            <button
              key={p.key}
              className={`side-nav-item ${page === p.key ? 'active' : ''}`}
              onClick={() => setPage(p.key)}
              title={p.label}
            >
              <span className="side-nav-icon">
                {renderNavIcon(p.key)}
              </span>
              {!sidebarCollapsed && <span className="side-nav-label">{p.label}</span>}
            </button>
          ))}
        </div>
        {currentAccount && !sidebarCollapsed && (
          <div className="side-nav-footer">
            <div className="current-account-badge">
              <span className="current-account-email">{currentAccount.email}</span>
              <span className={`plan-badge ${(currentAccount.plan_type || '').toLowerCase()}`}>
                {currentAccount.plan_type || 'FREE'}
              </span>
            </div>
          </div>
        )}
      </nav>
      <main className="main-content">
        {renderPage()}
      </main>
    </div>
  );
}

export { App };
