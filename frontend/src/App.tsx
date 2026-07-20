import React, { useEffect, useState } from 'react';
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
import { useI18n } from './i18n';

/* ── tab 类型 ── */
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

/* ── 分类定义 ── */
interface CategoryDef {
  icon: string;
  catKey: string;        // i18n key
  tabs: Tab[];
}

const CATEGORIES: CategoryDef[] = [
  { icon: '⚙️', catKey: 'cat.system',  tabs: ['chat', 'modules', 'vault'] },
  { icon: '🖥️', catKey: 'cat.infrastructure', tabs: ['hosts', 'topo', 'monitor'] },
  { icon: '🛡️', catKey: 'cat.security', tabs: ['security', 'fim', 'baseline'] },
  { icon: '🤖', catKey: 'cat.automation', tabs: ['scheduler', 'runbook', 'filesync'] },
  { icon: '🔔', catKey: 'cat.monitor', tabs: ['escalation', 'health'] },
  { icon: '🧠', catKey: 'cat.intelligence', tabs: ['advisor'] },
  { icon: '🔗', catKey: 'cat.integration', tabs: ['webhook', 'config', 'knowledge'] },
];

/* ── 底部操作 ── */
function AppShell() {
  const [tab, setTab] = React.useState<Tab>('modules');
  const { token, username, logout } = useAuthStore();
  const { isUnlocked, checkStatus } = useVaultStore();
  const { isDark, toggleDark } = useTheme();
  const { t, lang, setLang } = useI18n();
  const navigate = useNavigate();
  const location = useLocation();

  /* 侧边栏折叠状态（每个分类默认展开） */
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

  useEffect(() => {
    const path = location.pathname.slice(1) as Tab;
    if ((ALL_TABS as readonly string[]).includes(path)) {
      setTab(path);
      /* 激活的 tab 所在分类自动展开 */
      for (const cat of CATEGORIES) {
        if (cat.tabs.includes(path)) {
          setCollapsed(prev => ({ ...prev, [cat.catKey]: false }));
        }
      }
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

  const toggleCat = (catKey: string) => {
    setCollapsed(prev => ({ ...prev, [catKey]: !prev[catKey] }));
  };

  const vaultIcon = isUnlocked ? '🔓' : ICONS.vault;

  return (
    <div className="flex h-screen overflow-hidden bg-md-background">
      {/* ── 桌面侧边栏（分类可折叠） ── */}
      <aside className="hidden md:flex flex-col w-72 py-3 gap-0 bg-md-surface-container border-r border-md-outline-variant overflow-y-auto">
        {/* Logo */}
        <div className="flex items-center gap-3 px-4 mb-3">
          <div className="w-9 h-9 rounded-md-xl bg-md-primary flex items-center justify-center text-md-on-primary text-sm font-bold">
            OP
          </div>
          <span className="text-title-small font-medium text-md-on-surface">{t('app.name')}</span>
        </div>

        {/* 分类列表 */}
        <nav className="flex-1 space-y-0">
          {CATEGORIES.map(cat => {
            const isCollapsed = collapsed[cat.catKey] ?? false;
            return (
              <div key={cat.catKey}>
                {/* 分类头部（可点击折叠） */}
                <button
                  onClick={() => toggleCat(cat.catKey)}
                  className="flex items-center gap-2 w-full px-4 py-2 text-xs font-semibold uppercase tracking-wider text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors"
                >
                  <span className="text-base">{cat.icon}</span>
                  <span className="flex-1 text-left">{t(cat.catKey)}</span>
                  <svg
                    className={`w-4 h-4 transition-transform duration-200 ${isCollapsed ? '' : 'rotate-180'}`}
                    fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}
                  >
                    <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
                  </svg>
                </button>
                {/* Tab 列表 */}
                {!isCollapsed && (
                  <div className="pb-1">
                    {cat.tabs.map(key => (
                      <button
                        key={key}
                        onClick={() => navigateTo(key)}
                        className={`flex items-center gap-3 w-full pl-10 pr-4 py-2 text-sm font-medium transition-all duration-150 rounded-md-lg
                          ${tab === key
                            ? 'bg-md-secondary-container text-md-on-secondary-container'
                            : 'text-md-on-surface-variant hover:bg-md-surface-container-high'}`}
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

        {/* 底部操作区 */}
        <div className="border-t border-md-outline-variant pt-2 mt-2 space-y-0">
          {/* 语言切换 */}
          <button
            onClick={() => setLang(lang === 'zh' ? 'en' : 'zh')}
            className="flex items-center gap-3 w-full pl-4 pr-4 py-2 text-sm font-medium text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors rounded-md-lg"
          >
            <span className="text-lg">🌐</span>
            <span>{t('lang.' + lang)}</span>
          </button>

          {/* 深色/浅色 */}
          <button
            onClick={toggleDark}
            className="flex items-center gap-3 w-full pl-4 pr-4 py-2 text-sm font-medium text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors rounded-md-lg"
          >
            <span className="text-lg">{isDark ? '☀️' : '🌙'}</span>
            <span>{isDark ? t('nav.light') : t('nav.dark')}</span>
          </button>

          {/* 退出 */}
          <button
            onClick={logout}
            className="flex items-center gap-3 w-full pl-4 pr-4 py-2 text-sm font-medium text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors rounded-md-lg"
          >
            <span className="text-lg">🚪</span>
            <span>{t('nav.logout')}</span>
          </button>
        </div>
      </aside>

      {/* ── 主区域 ── */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* App Bar */}
        <header className="h-16 flex items-center justify-between px-4 sm:px-6 bg-md-surface-container/80 backdrop-blur-md border-b border-md-outline-variant sticky top-0 z-10">
          <h1 className="text-title-large font-medium text-md-on-surface">{t('title.' + tab)}</h1>
          <div className="flex items-center gap-3">
            <span className="text-sm text-md-on-surface-variant hidden sm:inline">{username}</span>
            <ThemePicker />
            <button
              onClick={toggleDark}
              className="w-9 h-9 rounded-md-full flex items-center justify-center hover:bg-md-surface-container-high transition-colors"
              title={isDark ? t('nav.light') : t('nav.dark')}
            >
              <span>{isDark ? '☀️' : '🌙'}</span>
            </button>
            <button
              onClick={() => setLang(lang === 'zh' ? 'en' : 'zh')}
              className="hidden md:flex w-9 h-9 rounded-md-full items-center justify-center hover:bg-md-surface-container-high transition-colors text-sm font-medium"
              title={lang === 'zh' ? 'English' : '中文'}
            >
              <span>{lang === 'zh' ? 'EN' : '中'}</span>
            </button>
            <button
              onClick={logout}
              className="hidden md:flex w-9 h-9 rounded-md-full items-center justify-center hover:bg-md-surface-container-high transition-colors"
              title={t('nav.logout')}
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

      {/* ── 底部导航（移动端） ── */}
      <nav className="md:hidden fixed bottom-0 left-0 right-0 h-16 bg-md-surface-container border-t border-md-outline-variant flex items-center justify-around px-2 z-20">
        {MOBILE_TABS.map(key => (
          <button
            key={key}
            onClick={() => navigateTo(key)}
            className={`flex flex-col items-center gap-0.5 py-1 px-3 rounded-md-md text-[11px] font-medium transition-colors
              ${tab === key ? 'text-md-primary' : 'text-md-on-surface-variant'}`}
          >
            <span className="text-xl">{ICONS[key]}</span>
            <span>{t('tab.' + key)}</span>
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
        <Route path="/login" element={token ? <Navigate to="/modules" replace /> : <LoginPage />} />
        <Route path="/*" element={token ? <AppShell /> : <Navigate to="/login" replace />} />
      </Routes>
    </ErrorBoundary>
  );
}
