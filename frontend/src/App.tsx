import { useEffect, useState } from 'react';
import { Routes, Route, Navigate, useNavigate, useLocation } from 'react-router-dom';
import { Dashboard } from './components/Dashboard';
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
import { useI18n } from './i18n';
import { cn } from './lib/cn';

/* ── tab 类型 ── */
type Tab =
  | 'dashboard' | 'chat' | 'modules' | 'hosts' | 'vault' | 'security' | 'health'
  | 'topo' | 'monitor' | 'escalation' | 'fim' | 'baseline' | 'runbook'
  | 'knowledge' | 'config' | 'webhook' | 'scheduler' | 'filesync' | 'advisor';

const ALL_TABS: Tab[] = [
  'dashboard', 'chat', 'modules', 'hosts', 'vault', 'security', 'health',
  'topo', 'monitor', 'escalation', 'fim', 'baseline', 'runbook',
  'knowledge', 'config', 'webhook', 'scheduler', 'filesync', 'advisor',
];

const MOBILE_TABS: Tab[] = [
  'dashboard', 'chat', 'hosts', 'vault', 'security', 'health',
];

const ICONS: Record<Tab, string> = {
  dashboard: '📊',
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

/* ── 扁平化分类（无二级嵌套的独立分类） */
const SIDEBAR_ITEMS: { icon: string; catKey: string; tabs: Tab[] }[] = [
  { icon: '📊', catKey: 'cat.dashboard', tabs: ['dashboard'] },
  { icon: '💬', catKey: 'cat.system', tabs: ['chat', 'modules', 'vault'] },
  { icon: '🖥️', catKey: 'cat.infrastructure', tabs: ['hosts', 'topo', 'monitor'] },
  { icon: '🛡️', catKey: 'cat.security', tabs: ['security', 'fim', 'baseline'] },
  { icon: '🤖', catKey: 'cat.automation', tabs: ['scheduler', 'runbook', 'filesync'] },
  { icon: '🔔', catKey: 'cat.monitor', tabs: ['escalation', 'health'] },
  { icon: '🧠', catKey: 'cat.intelligence', tabs: ['advisor'] },
  { icon: '🔗', catKey: 'cat.integration', tabs: ['webhook', 'config', 'knowledge'] },
];

/* ── 侧边栏分类名称翻译 ── */
const CAT_KEY_LABELS: Record<string, string> = {
  'cat.dashboard': '',
  'cat.system': '系统管理',
  'cat.infrastructure': '基础设施',
  'cat.security': '安全合规',
  'cat.automation': '自动化',
  'cat.monitor': '监控告警',
  'cat.intelligence': '智能分析',
  'cat.integration': '集成管理',
};

/* ── AppShell ── */
function AppShell() {
  const [tab, setTab] = useState<Tab>('dashboard');
  const { token, username, logout } = useAuthStore();
  const { isUnlocked, checkStatus } = useVaultStore();
  const { isDark, toggleDark } = useTheme();
  const { t, lang, setLang } = useI18n();
  const navigate = useNavigate();
  const location = useLocation();

  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

  useEffect(() => {
    const path = location.pathname.slice(1) as Tab;
    if ((ALL_TABS as readonly string[]).includes(path)) {
      setTab(path);
      for (const item of SIDEBAR_ITEMS) {
        if (item.tabs.includes(path)) {
          setCollapsed(prev => ({ ...prev, [item.catKey]: false }));
        }
      }
    }
  }, [location.pathname]);

  useEffect(() => {
    if (token) checkStatus();
  }, [token, checkStatus]);

  const navigateTo = (key: Tab) => {
    setTab(key);
    navigate('/' + key);
  };

  const toggleCat = (catKey: string) => {
    setCollapsed(prev => ({ ...prev, [catKey]: !prev[catKey] }));
  };

  const vaultIcon = isUnlocked ? '🔓' : ICONS.vault;

  const renderContent = () => {
    switch (tab) {
      case 'dashboard': return <Dashboard />;
      case 'chat': return <AgentChat />;
      case 'modules': return <ModuleBrowser />;
      case 'hosts': return <HostsPage />;
      case 'vault': return <VaultPage />;
      case 'security': return <SecurityPage />;
      case 'health': return <HealthDashboard />;
      case 'topo': return <TopologyPage />;
      case 'monitor': return <MonitorPage />;
      case 'escalation': return <EscalationPage />;
      case 'fim': return <FIMPage />;
      case 'baseline': return <BaselinePage />;
      case 'runbook': return <RunbookPage />;
      case 'knowledge': return <KnowledgePage />;
      case 'config': return <ConfigPage />;
      case 'webhook': return <WebhookPage />;
      case 'scheduler': return <SchedulerPage />;
      case 'filesync': return <FileSyncPage />;
      case 'advisor': return <AdvisorPage />;
      default: return <Dashboard />;
    }
  };

  return (
    <div className="flex h-screen overflow-hidden bg-md-background">
      {/* ── 桌面侧边栏 ── */}
      <aside className="hidden md:flex flex-col w-64 py-3 gap-0 bg-md-surface-container/95 backdrop-blur-xl border-r border-md-outline-variant overflow-y-auto shrink-0">
        {/* Logo */}
        <div className="flex items-center gap-3 px-5 mb-4">
          <div className="w-9 h-9 rounded-md-xl bg-gradient-to-br from-md-primary to-md-tertiary flex items-center justify-center text-md-on-primary text-sm font-bold shadow-md-2">
            OP
          </div>
          <div>
            <span className="text-title-medium font-semibold text-md-on-surface">{t('app.name')}</span>
            <p className="text-label-medium text-md-on-surface-variant leading-none mt-0.5">v2.0</p>
          </div>
        </div>

        {/* 分类导航 */}
        <nav className="flex-1 space-y-1 px-2">
          {SIDEBAR_ITEMS.map(item => {
            /* dashboard 单独一行，无折叠 */
            if (item.catKey === 'cat.dashboard') {
              const key = item.tabs[0];
              return (
                <button
                  key={key}
                  onClick={() => navigateTo(key)}
                  className={cn(
                    'flex items-center gap-3 w-full px-3 py-2.5 text-sm font-medium transition-all duration-200 rounded-md-lg',
                    tab === key
                      ? 'glass-card text-md-primary shadow-sm'
                      : 'text-md-on-surface-variant hover:glass-card hover:text-md-on-surface',
                  )}
                >
                  <span className="text-xl">{ICONS[key]}</span>
                  <span>{t('title.dashboard')}</span>
                </button>
              );
            }

            const isCollapsed = collapsed[item.catKey] ?? false;
            return (
              <div key={item.catKey}>
                <button
                  onClick={() => toggleCat(item.catKey)}
                  className="flex items-center gap-2 w-full px-3 py-1.5 text-[11px] font-semibold uppercase tracking-widest text-md-on-surface-variant/60 hover:text-md-on-surface-variant transition-colors"
                >
                  <span className="text-sm">{item.icon}</span>
                  <span className="flex-1 text-left">{CAT_KEY_LABELS[item.catKey]}</span>
                  <svg
                    className={cn('w-3 h-3 transition-transform duration-200', isCollapsed ? '' : 'rotate-180')}
                    fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}
                  >
                    <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
                  </svg>
                </button>
                {!isCollapsed && (
                  <div className="pb-0.5 space-y-0.5">
                    {item.tabs.map(key => (
                      <button
                        key={key}
                        onClick={() => navigateTo(key)}
                        className={cn(
                          'flex items-center gap-3 w-full pl-9 pr-3 py-2 text-sm font-medium transition-all duration-150 rounded-md-lg',
                          tab === key
                            ? 'glass-card text-md-on-surface shadow-sm'
                            : 'text-md-on-surface-variant hover:glass-card hover:text-md-on-surface',
                        )}
                      >
                        <span className="text-lg">{key === 'vault' ? vaultIcon : ICONS[key]}</span>
                        <span>{t('tab.' + key)}</span>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            );
          })}
        </nav>

        {/* 底部 */}
        <div className="border-t border-md-outline-variant/50 pt-2 mt-2 px-2 space-y-0.5">
          <button
            onClick={() => setLang(lang === 'zh' ? 'en' : 'zh')}
            className="flex items-center gap-3 w-full px-3 py-2 text-sm font-medium text-md-on-surface-variant hover:glass-card transition-all rounded-md-lg"
          >
            <span className="text-lg">🌐</span>
            <span>{t('lang.' + lang)}</span>
          </button>
          <button
            onClick={toggleDark}
            className="flex items-center gap-3 w-full px-3 py-2 text-sm font-medium text-md-on-surface-variant hover:glass-card transition-all rounded-md-lg"
          >
            <span className="text-lg">{isDark ? '☀️' : '🌙'}</span>
            <span>{isDark ? t('nav.light') : t('nav.dark')}</span>
          </button>
          <button
            onClick={logout}
            className="flex items-center gap-3 w-full px-3 py-2 text-sm font-medium text-md-on-surface-variant hover:glass-card transition-all rounded-md-lg"
          >
            <span className="text-lg">🚪</span>
            <span>{t('nav.logout')}</span>
          </button>
        </div>
      </aside>

      {/* ── 主区域 ── */}
      <div className="flex-1 flex flex-col min-w-0">
        <header className="h-16 flex items-center justify-between px-4 sm:px-6 bg-md-surface-container/70 backdrop-blur-xl border-b border-md-outline-variant/50 sticky top-0 z-10">
          <h1 className="text-title-large font-semibold text-md-on-surface">
            {tab === 'dashboard' ? (
              <>
                <span className="gradient-text">数据大屏</span>
                <span className="text-body-medium text-md-on-surface-variant ml-3 font-normal">实时运维总览</span>
              </>
            ) : t('title.' + tab)}
          </h1>
          <div className="flex items-center gap-2">
            <span className="text-sm text-md-on-surface-variant hidden sm:inline mr-1">{username}</span>
            <ThemePicker />
            <button
              onClick={toggleDark}
              className="w-9 h-9 rounded-md-full flex items-center justify-center hover:glass-card transition-all"
              title={isDark ? t('nav.light') : t('nav.dark')}
            >
              <span>{isDark ? '☀️' : '🌙'}</span>
            </button>
            <button
              onClick={() => setLang(lang === 'zh' ? 'en' : 'zh')}
              className="hidden md:flex w-9 h-9 rounded-md-full items-center justify-center hover:glass-card transition-all text-sm font-medium"
              title={lang === 'zh' ? 'English' : '中文'}
            >
              <span className="text-xs font-bold">{lang === 'zh' ? 'EN' : '中'}</span>
            </button>
            <button
              onClick={logout}
              className="hidden md:flex w-9 h-9 rounded-md-full items-center justify-center hover:glass-card transition-all"
              title={t('nav.logout')}
            >
              🚪
            </button>
          </div>
        </header>

        <main className="flex-1 overflow-auto p-4 sm:p-6 pb-20 md:pb-6">
          <ErrorBoundary key={tab}>
            {renderContent()}
          </ErrorBoundary>
        </main>
      </div>

      {/* ── 移动端底部导航 ── */}
      <nav className="md:hidden fixed bottom-0 left-0 right-0 h-16 bg-md-surface-container/95 backdrop-blur-xl border-t border-md-outline-variant/50 flex items-center justify-around px-2 z-20">
        {MOBILE_TABS.map(key => (
          <button
            key={key}
            onClick={() => navigateTo(key)}
            className={cn(
              'flex flex-col items-center gap-0.5 py-1 px-3 rounded-md-md text-[11px] font-medium transition-colors',
              tab === key ? 'text-md-primary' : 'text-md-on-surface-variant',
            )}
          >
            <span className="text-xl">{ICONS[key]}</span>
            <span>{
              key === 'dashboard' ? '概览'
              : t('tab.' + key)
            }</span>
          </button>
        ))}
      </nav>
    </div>
  );
}

/* ── 根组件 ── */
export function App() {
  const { token } = useAuthStore();

  return (
    <ErrorBoundary>
      <Routes>
        <Route path="/login" element={token ? <Navigate to="/dashboard" replace /> : <LoginPage />} />
        <Route path="/*" element={token ? <AppShell /> : <Navigate to="/login" replace />} />
      </Routes>
    </ErrorBoundary>
  );
}
