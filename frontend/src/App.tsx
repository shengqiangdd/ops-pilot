import React, { useEffect } from 'react';
import { Routes, Route, Navigate, useNavigate, useLocation } from 'react-router-dom';
import { ModuleBrowser } from './components/ModuleBrowser';
import { HealthDashboard } from './components/HealthDashboard';
import { AgentChat } from './components/AgentChat';
import { HostsPage } from './pages/Hosts';
import { VaultPage } from './pages/Vault';
import { LoginPage } from './pages/Login';
import { SecurityPage } from './pages/Security';
import { TopologyPage } from './pages/Topology';
import { MonitorPage } from './pages/Monitor';
import { EscalationPage } from './pages/Escalation';
import { FIMPage } from './pages/FIM';
import { BaselinePage } from './pages/Baseline';
import { RunbookPage } from './pages/Runbook';
import { KnowledgePage } from './pages/Knowledge';
import { ConfigPage } from './pages/Config';
import { WebhookPage } from './pages/Webhook';
import { SchedulerPage } from './pages/Scheduler';
import { FileSyncPage } from './pages/FileSync';
import { AdvisorPage } from './pages/Advisor';
import { useAuthStore } from './stores/useAuthStore';
import { useVaultStore } from './stores/useVaultStore';
import { ErrorBoundary } from './components/ErrorBoundary';
import { useTheme } from './components/ThemeProvider';
import { ThemePicker } from './components/ThemePicker';

type Tab =
  | 'chat' | 'modules' | 'hosts' | 'vault' | 'security' | 'health'
  | 'topo' | 'monitor' | 'escalation' | 'fim' | 'baseline' | 'runbook'
  | 'knowledge' | 'config' | 'webhook' | 'scheduler' | 'filesync' | 'advisor';

const ALL_TABS: Tab[] = [
  'chat', 'modules', 'hosts', 'vault', 'security', 'health',
  'topo', 'monitor', 'escalation', 'fim', 'baseline', 'runbook',
  'knowledge', 'config', 'webhook', 'scheduler', 'filesync', 'advisor',
];

const MOBILE_TABS: Tab[] = [
  'chat', 'modules', 'hosts', 'vault', 'security', 'health', 'topo', 'monitor', 'escalation',
];

const ICONS: Record<Tab, string> = {
  chat: '💬',
  modules: '🧩',
  hosts: '🖥️',
  vault: '🔑',
  security: '🛡️',
  health: '❤️',
  topo: '🗺️',
  monitor: '📊',
  escalation: '🔔',
  fim: '🔍',
  baseline: '✅',
  runbook: '📋',
  knowledge: '📚',
  config: '⚙️',
  webhook: '🔌',
  scheduler: '⏰',
  filesync: '📁',
  advisor: '💡',
};

const SHORT_LABELS: Record<Tab, string> = {
  chat: 'Chat',
  modules: 'Mods',
  hosts: 'Hosts',
  vault: 'Vault',
  security: 'Sec',
  health: 'Health',
  topo: 'Topo',
  monitor: 'Monitor',
  escalation: 'Alert',
  fim: 'FIM',
  baseline: 'Base',
  runbook: 'Run',
  knowledge: 'Know',
  config: 'Config',
  webhook: 'Hook',
  scheduler: 'Cron',
  filesync: 'Sync',
  advisor: 'Adv',
};

const TAB_TITLES: Record<Tab, string> = {
  chat: 'Agent Chat',
  modules: 'Modules',
  hosts: 'Hosts',
  vault: 'Vault',
  security: 'Security Scanning',
  health: 'Health Dashboard',
  topo: 'Topology',
  monitor: 'Monitor',
  escalation: 'Escalation',
  fim: 'File Integrity',
  baseline: 'Baseline',
  runbook: 'Runbook',
  knowledge: 'Knowledge Base',
  config: 'Configuration',
  webhook: 'Webhooks',
  scheduler: 'Scheduler',
  filesync: 'File Sync',
  advisor: 'Advisor',
};

function AppShell() {
  const [tab, setTab] = React.useState<Tab>('modules');
  const { token, username, logout } = useAuthStore();
  const { isUnlocked, checkStatus } = useVaultStore();
  const { isDark, toggleDark } = useTheme();
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    const path = location.pathname.slice(1) as Tab;
    if ((ALL_TABS as readonly string[]).includes(path)) {
      setTab(path);
    }
  }, [location.pathname]);

  useEffect(() => {
    if (token) {
      checkStatus();
    }
  }, [token, checkStatus]);

  const navigateTo = (key: Tab) => {
    setTab(key);
    navigate('/' + key);
  };

  const vaultIcon = isUnlocked ? '🔓' : ICONS.vault;

  return (
    <div className="flex h-screen overflow-hidden bg-md-background">
      {/* Navigation Rail - Desktop */}
      <aside className="hidden md:flex flex-col w-20 items-center py-4 gap-2 bg-md-surface-container border-r border-md-outline-variant">
        <div className="w-10 h-10 rounded-md-xl bg-md-primary flex items-center justify-center text-md-on-primary text-lg font-bold mb-4">OP</div>
        {ALL_TABS.map(key => (
          <button
            key={key}
            onClick={() => navigateTo(key)}
            className={`flex flex-col items-center gap-1 w-16 py-2 rounded-md-lg text-xs font-medium transition-all duration-200
              ${tab === key ? 'bg-md-secondary-container text-md-on-secondary-container' : 'text-md-on-surface-variant hover:bg-md-surface-container-high'}`}
          >
            <span className="text-xl">{key === 'vault' ? vaultIcon : ICONS[key]}</span>
            <span className="truncate w-full text-center">{SHORT_LABELS[key]}</span>
          </button>
        ))}
        <div className="flex-1" />
        <button
          onClick={toggleDark}
          className="flex flex-col items-center gap-1 w-16 py-2 rounded-md-lg text-xs font-medium text-md-on-surface-variant hover:bg-md-surface-container-high transition-all"
        >
          <span className="text-xl">{isDark ? '☀️' : '🌙'}</span>
          <span>{isDark ? 'Light' : 'Dark'}</span>
        </button>
        <button
          onClick={logout}
          className="flex flex-col items-center gap-1 w-16 py-2 rounded-md-lg text-xs font-medium text-md-on-surface-variant hover:bg-md-surface-container-high transition-all"
        >
          <span className="text-xl">🚪</span>
          <span>Logout</span>
        </button>
      </aside>

      {/* Main Area */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* App Bar */}
        <header className="h-16 flex items-center justify-between px-4 sm:px-6 bg-md-surface-container/80 backdrop-blur-md border-b border-md-outline-variant sticky top-0 z-10">
          <h1 className="text-title-large font-medium text-md-on-surface">{TAB_TITLES[tab]}</h1>
          <div className="flex items-center gap-3">
            <span className="text-sm text-md-on-surface-variant hidden sm:inline">{username}</span>
            <ThemePicker />
            <button
              onClick={toggleDark}
              className="w-9 h-9 rounded-md-full flex items-center justify-center hover:bg-md-surface-container-high transition-colors"
            >
              {isDark ? '☀️' : '🌙'}
            </button>
            <button
              onClick={logout}
              className="hidden md:flex w-9 h-9 rounded-md-full items-center justify-center hover:bg-md-surface-container-high transition-colors"
              title="Logout"
            >
              🚪
            </button>
          </div>
        </header>

        {/* Content */}
        <main className="flex-1 overflow-auto p-4 sm:p-6 animate-fade-in pb-20 md:pb-6">
          <ErrorBoundary key={tab}>
            {tab === 'chat' && <AgentChat />}
            {tab === 'modules' && <ModuleBrowser />}
            {tab === 'hosts' && <HostsPage />}
            {tab === 'vault' && <VaultPage />}
            {tab === 'security' && <SecurityPage />}
            {tab === 'health' && <HealthDashboard />}
            {tab === 'topo' && <TopologyPage />}
            {tab === 'monitor' && <MonitorPage />}
            {tab === 'escalation' && <EscalationPage />}
            {tab === 'fim' && <FIMPage />}
            {tab === 'baseline' && <BaselinePage />}
            {tab === 'runbook' && <RunbookPage />}
            {tab === 'knowledge' && <KnowledgePage />}
            {tab === 'config' && <ConfigPage />}
            {tab === 'webhook' && <WebhookPage />}
            {tab === 'scheduler' && <SchedulerPage />}
            {tab === 'filesync' && <FileSyncPage />}
            {tab === 'advisor' && <AdvisorPage />}
          </ErrorBoundary>
        </main>
      </div>

      {/* Bottom Navigation - Mobile */}
      <nav className="md:hidden fixed bottom-0 left-0 right-0 h-16 bg-md-surface-container border-t border-md-outline-variant flex items-center justify-around px-2 z-20">
        {MOBILE_TABS.map(key => (
          <button
            key={key}
            onClick={() => navigateTo(key)}
            className={`flex flex-col items-center gap-0.5 py-1 px-3 rounded-md-md text-[11px] font-medium transition-colors
              ${tab === key ? 'text-md-primary' : 'text-md-on-surface-variant'}`}
          >
            <span className="text-xl">{ICONS[key]}</span>
            <span>{SHORT_LABELS[key]}</span>
          </button>
        ))}
      </nav>
    </div>
  );
}

export function App() {
  const { token } = useAuthStore();

  return (
    <ErrorBoundary>
      <Routes>
        <Route path="/login" element={token ? <Navigate to="/modules" replace /> : <LoginPage />} />
        <Route path="/*" element={token ? <AppShell /> : <Navigate to="/login" replace />} />
      </Routes>
    </ErrorBoundary>
  );
}
