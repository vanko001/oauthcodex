export type CodexTab = 'overview' | 'providers' | 'wakeup' | 'instances' | 'sessions';

interface CodexOverviewTabsHeaderProps {
  active: CodexTab;
  onTabChange?: (tab: CodexTab) => void;
  tabs?: CodexTab[];
}

const DEFAULT_TABS: CodexTab[] = ['overview', 'providers', 'wakeup', 'instances', 'sessions'];

const TAB_LABELS: Record<CodexTab, string> = {
  overview: 'Overview',
  providers: 'Providers',
  wakeup: 'Wakeup',
  instances: 'Instances',
  sessions: 'Sessions',
};

export function CodexOverviewTabsHeader({
  active,
  onTabChange,
  tabs = DEFAULT_TABS,
}: CodexOverviewTabsHeaderProps) {
  return (
    <div className="tabs-header">
      {tabs.map((tab) => (
        <button
          key={tab}
          className={`tab-btn ${active === tab ? 'active' : ''}`}
          onClick={() => onTabChange?.(tab)}
        >
          {TAB_LABELS[tab]}
        </button>
      ))}
    </div>
  );
}
