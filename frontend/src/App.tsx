import { Suspense, lazy, useEffect, useState } from 'react';
import { Routes, Route, Navigate, useNavigate, useLocation } from 'react-router-dom';
import { AnimatePresence, motion } from 'framer-motion';
import { Dashboard } from './components/Dashboard';
import { ModuleBrowser } from './components/ModuleBrowser';
import { HealthDashboard } from './components/HealthDashboard';
import { AgentChat } from './components/AgentChat';
import { LoginPage } from './pages/Login';
import { useAuthStore } from './stores/useAuthStore';
import { useVaultStore } from './stores/useVaultStore';
import { ErrorBoundary, installGlobalErrorListener } from './components/ErrorBoundary';
import { useKeyboardShortcuts, useNavigationShortcuts } from './hooks/useKeyboardShortcuts';
import { ShortcutHelp } from './components/ShortcutHelp';
import { useTheme } from './components/ThemeProvider';
import { ThemePicker } from './components/ThemePicker';
import { GlobalSearch } from './components/GlobalSearch';
import { useI18n } from './i18n';
import { cn } from './lib/cn';
import { ChartPageSkeleton } from './components/PageSkeleton';

// Lazy-loaded page components for code splitting
const HostsPage = lazy(() => import('./pages/Hosts').then(m => ({ default: m.HostsPage })));
const VaultPage = lazy(() => import('./pages/Vault').then(m => ({ default: m.VaultPage })));
const SecurityPage = lazy(() => import('./pages/Security').then(m => ({ default: m.SecurityPage })));
const TopologyPage = lazy(() => import('./pages/Topology').then(m => ({ default: m.TopologyPage })));
const MonitorPage = lazy(() => import('./pages/Monitor').then(m => ({ default: m.MonitorPage })));
const EscalationPage = lazy(() => import('./pages/Escalation').then(m => ({ default: m.EscalationPage })));
const FIMPage = lazy(() => import('./pages/FIM').then(m => ({ default: m.FIMPage })));
const BaselinePage = lazy(() => import('./pages/Baseline').then(m => ({ default: m.BaselinePage })));
const RunbookPage = lazy(() => import('./pages/Runbook').then(m => ({ default: m.RunbookPage })));
const KnowledgePage = lazy(() => import('./pages/Knowledge').then(m => ({ default: m.KnowledgePage })));
const ConfigPage = lazy(() => import('./pages/Config').then(m => ({ default: m.ConfigPage })));
const WebhookPage = lazy(() => import('./pages/Webhook').then(m => ({ default: m.WebhookPage })));
const SchedulerPage = lazy(() => import('./pages/Scheduler').then(m => ({ default: m.SchedulerPage })));
const FileSyncPage = lazy(() => import('./pages/FileSync').then(m => ({ default: m.FileSyncPage })));
const AdvisorPage = lazy(() => import('./pages/Advisor').then(m => ({ default: m.AdvisorPage })));
const TerminalPage = lazy(() => import('./pages/Terminal').then(m => ({ default: m.TerminalPage })));
const AuditLogPage = lazy(() => import('./pages/AuditLog').then(m => ({ default: m.AuditLogPage })));
const UsersPage = lazy(() => import('./pages/Users').then(m => ({ default: m.UsersPage })));
const AlertRulesPage = lazy(() => import('./pages/AlertRules').then(m => ({ default: m.AlertRulesPage })));
const AlertHistoryPage = lazy(() => import('./pages/AlertHistory').then(m => ({ default: m.AlertHistoryPage })));
const NotificationChannelsPage = lazy(() => import('./pages/NotificationChannels').then(m => ({ default: m.NotificationChannelsPage })));
const CMDBPage = lazy(() => import('./pages/CMDB').then(m => ({ default: m.CMDBPage })));
const TimelinePage = lazy(() => import('./pages/Timeline').then(m => ({ default: m.TimelinePage })));
const CICDPage = lazy(() => import('./pages/CICD').then(m => ({ default: m.CICDPage })));
const MetricsVizPage = lazy(() => import('./pages/MetricsViz').then(m => ({ default: m.MetricsVizPage })));
const JobsPage = lazy(() => import('./pages/Jobs').then(m => ({ default: m.JobsPage })));
const DiagnosticsPage = lazy(() => import('./pages/Diagnostics').then(m => ({ default: m.DiagnosticsPage })));
const ReportsPage = lazy(() => import('./pages/ReportsView').then(m => ({ default: m.ReportsViewPage })));
const BackupRestorePage = lazy(() => import('./pages/BackupRestore').then(m => ({ default: m.BackupRestorePage })));
const IncidentsPage = lazy(() => import('./pages/Incidents').then(m => ({ default: m.IncidentsPage })));
const VulnerabilitiesPage = lazy(() => import('./pages/Vulnerabilities').then(m => ({ default: m.VulnerabilitiesPage })));
const PredictionsPage = lazy(() => import('./pages/Predictions').then(m => ({ default: m.PredictionsPage })));
const SLOsPage = lazy(() => import('./pages/SLOs').then(m => ({ default: m.SLOsPage })));
const SOARPage = lazy(() => import('./pages/SOAR').then(m => ({ default: m.SOARPage })));
const RemediationPage = lazy(() => import('./pages/Remediation').then(m => ({ default: m.RemediationPage })));
const SecretsScanPage = lazy(() => import('./pages/SecretsScan').then(m => ({ default: m.SecretsScanPage })));
const CompliancePage = lazy(() => import('./pages/Compliance').then(m => ({ default: m.CompliancePage })));
const ThreatsPage = lazy(() => import('./pages/Threats').then(m => ({ default: m.ThreatsPage })));
const ChangeAnalysisPage = lazy(() => import('./pages/ChangeAnalysis').then(m => ({ default: m.ChangeAnalysisPage })));
const LogIntelligencePage = lazy(() => import('./pages/LogIntelligence').then(m => ({ default: m.LogIntelligencePage })));
const OnCallPage = lazy(() => import('./pages/OnCall').then(m => ({ default: m.OnCallPage })));
const ChaosPage = lazy(() => import('./pages/Chaos').then(m => ({ default: m.ChaosPage })));
const FinOpsPage = lazy(() => import('./pages/FinOps').then(m => ({ default: m.FinOpsPage })));
const APMPage = lazy(() => import('./pages/APM').then(m => ({ default: m.APMPage })));
const AdvisorChatPage = lazy(() => import('./pages/AdvisorChat').then(m => ({ default: m.AdvisorChat })));
const OpsDashboardPage = lazy(() => import('./pages/OpsDashboard').then(m => ({ default: m.OpsDashboard })));
const ChangeRiskPage = lazy(() => import('./pages/ChangeRisk').then(m => ({ default: m.ChangeRiskPage })));
const InspectionPage = lazy(() => import('./pages/Inspection').then(m => ({ default: m.InspectionPage })));
const IdsPage = lazy(() => import('./pages/Ids').then(m => ({ default: m.IdsPage })));
const ContainerSecPage = lazy(() => import('./pages/ContainerSec').then(m => ({ default: m.ContainerSecPage })));
const ApiDocsPage = lazy(() => import('./pages/ApiDocs').then(m => ({ default: m.default })));
const AlertDiagnosisPage = lazy(() => import('./pages/AlertDiagnosis').then(m => ({ default: m.AlertDiagnosisPage })));
const GitOpsPage = lazy(() => import('./pages/GitOps').then(m => ({ default: m.GitOpsPage })));
const DashboardLayoutsPage = lazy(() => import('./pages/DashboardLayouts').then(m => ({ default: m.DashboardLayoutsPage })));
const AuditLogViewPage = lazy(() => import('./pages/AuditLogView').then(m => ({ default: m.AuditLogViewPage })));
const SessionReplayPage = lazy(() => import('./pages/SessionReplay').then(m => ({ default: m.SessionReplayPage })));
const RCAnalysisPage = lazy(() => import('./pages/RCAnalysis').then(m => ({ default: m.RCAnalysisPage })));
const ClusterManagerPage = lazy(() => import('./pages/ClusterManager').then(m => ({ default: m.ClusterManagerPage })));

/* ── Loading fallback ── */
function LoadingFallback() {
  return <div className="p-6"><ChartPageSkeleton /></div>;
}

/* ── tab 类型 ── */
type Tab =
  | 'dashboard' | 'chat' | 'modules' | 'hosts' | 'vault' | 'security' | 'health'
  | 'backup' | 'topo' | 'monitor' | 'escalation' | 'fim' | 'baseline' | 'runbook'
  | 'knowledge' | 'config' | 'webhook' | 'scheduler' | 'filesync' | 'advisor'
  | 'terminal' | 'audit' | 'users' | 'alert-rules' | 'alert-history' | 'channels' | 'cmdb' | 'timeline' | 'cicd' | 'metrics' | 'jobs' | 'diagnostics' | 'reports' | 'incidents' | 'vulnerabilities' | 'predictions' | 'slos' | 'soar' | 'remediation' | 'secrets-scan' | 'compliance' | 'threats' | 'change-analysis' | 'log-intel' | 'oncall' | 'chaos' | 'finops' | 'apm' | 'ops-dashboard' | 'change-risk' | 'inspection' | 'ids' | 'container-sec'
  | 'alert-diagnosis' | 'gitops' | 'dashboard-layouts' | 'audit-log-view'
  | 'session-replay' | 'rca-analysis' | 'clusters';

const ALL_TABS: Tab[] = [
  'ops-dashboard', 'dashboard', 'chat', 'modules', 'backup', 'hosts', 'vault', 'security', 'health',
  'topo', 'monitor', 'escalation', 'fim', 'baseline', 'runbook',
  'knowledge', 'config', 'webhook', 'scheduler', 'filesync', 'advisor',
  'terminal', 'audit', 'users', 'alert-rules', 'alert-history', 'channels', 'cmdb', 'timeline', 'cicd', 'metrics', 'jobs', 'diagnostics', 'reports', 'incidents', 'vulnerabilities', 'predictions', 'slos', 'soar', 'remediation', 'secrets-scan', 'compliance', 'threats', 'change-analysis', 'log-intel', 'oncall', 'chaos', 'finops', 'apm', 'change-risk', 'inspection', 'ids', 'container-sec',
  'alert-diagnosis', 'gitops', 'dashboard-layouts', 'audit-log-view',
  'session-replay', 'rca-analysis', 'clusters',
];

const MOBILE_TABS: Tab[] = [
  'dashboard', 'chat', 'hosts', 'vault', 'security', 'health',
];

const ICONS: Record<Tab, string> = {
  dashboard: '📊',
  'ops-dashboard': '📡',
  backup: '🔄',
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
  terminal: '⌨️',
  audit: '📋',
  users: '👥',
  'alert-rules': '📐',
  'alert-history': '📜',
  channels: '📢',
  cmdb: '🗄️',
  timeline: '📅',
  cicd: '🚀',
  metrics: '📈',
  jobs: '📋',
  diagnostics: '🩺',
  reports: '📄',
  incidents: '🚨',
  vulnerabilities: '🔓',
  predictions: '🔮',
  slos: '📊',
  soar: '🎯',
  remediation: '🔧',
  'secrets-scan': '🔐',
  compliance: '✅',
  threats: '⚠️',
  'change-analysis': '📝',
  'log-intel': '📊',
  oncall: '📅',
  chaos: '💥',
  finops: '💰',
  apm: '📈',
  'change-risk': '⚠️',
  inspection: '🔍',
  ids: '🛡️',
  'container-sec': '🐳',
  'alert-diagnosis': '🩺',
  gitops: '🔀',
  'dashboard-layouts': '📐',
  'audit-log-view': '📋',
  'session-replay': '⏪',
  'rca-analysis': '🔬',
  clusters: '🌐',
};

/* ── Tab role requirements ── */
type Role = 'admin' | 'operator' | 'viewer';
const TAB_ROLES: Record<Tab, Role[]> = {
  'ops-dashboard': ['viewer', 'operator', 'admin'],
  backup: ['admin'],
  dashboard: ['viewer', 'operator', 'admin'],
  chat: ['operator', 'admin'],
  modules: ['operator', 'admin'],
  hosts: ['operator', 'admin'],
  vault: ['operator', 'admin'],
  security: ['operator', 'admin'],
  health: ['viewer', 'operator', 'admin'],
  topo: ['viewer', 'operator', 'admin'],
  monitor: ['viewer', 'operator', 'admin'],
  escalation: ['operator', 'admin'],
  fim: ['operator', 'admin'],
  baseline: ['operator', 'admin'],
  runbook: ['operator', 'admin'],
  knowledge: ['viewer', 'operator', 'admin'],
  config: ['operator', 'admin'],
  webhook: ['operator', 'admin'],
  scheduler: ['operator', 'admin'],
  filesync: ['operator', 'admin'],
  advisor: ['operator', 'admin'],
  terminal: ['operator', 'admin'],
  audit: ['admin'],
  users: ['admin'],
  'alert-rules': ['operator', 'admin'],
  'alert-history': ['viewer', 'operator', 'admin'],
  channels: ['operator', 'admin'],
  cmdb: ['operator', 'admin'],
  timeline: ['viewer', 'operator', 'admin'],
  cicd: ['operator', 'admin'],
  metrics: ['viewer', 'operator', 'admin'],
  jobs: ['operator', 'admin'],
  diagnostics: ['operator', 'admin'],
  reports: ['viewer', 'operator', 'admin'],
  incidents: ['operator', 'admin'],
  vulnerabilities: ['operator', 'admin'],
  predictions: ['operator', 'admin'],
  slos: ['operator', 'admin'],
  soar: ['operator', 'admin'],
  remediation: ['operator', 'admin'],
  'secrets-scan': ['operator', 'admin'],
  compliance: ['operator', 'admin'],
  threats: ['operator', 'admin'],
  'change-analysis': ['operator', 'admin'],
  'log-intel': ['operator', 'admin'],
  oncall: ['operator', 'admin'],
  chaos: ['operator', 'admin'],
  finops: ['operator', 'admin'],
  apm: ['viewer', 'operator', 'admin'],
  'change-risk': ['operator', 'admin'],
  inspection: ['operator', 'admin'],
  ids: ['operator', 'admin'],
  'container-sec': ['operator', 'admin'],
  'alert-diagnosis': ['operator', 'admin'],
  gitops: ['admin'],
  'dashboard-layouts': ['operator', 'admin'],
  'audit-log-view': ['admin'],
  'session-replay': ['operator', 'admin'],
  'rca-analysis': ['operator', 'admin'],
  clusters: ['admin'],
};

const ROLE_HIERARCHY: Record<Role, number> = {
  admin: 3,
  operator: 2,
  viewer: 1,
};

function hasRequiredRole(userRole: Role | null, required: Role[]): boolean {
  if (!userRole) return false;
  const userLevel = ROLE_HIERARCHY[userRole];
  return required.some(r => userLevel >= ROLE_HIERARCHY[r]);
}

/* ── 扁平化分类（无二级嵌套的独立分类） */
const SIDEBAR_ITEMS: { icon: string; catKey: string; tabs: Tab[] }[] = [
  { icon: '📡', catKey: 'cat.overview', tabs: ['ops-dashboard'] },
  { icon: '💬', catKey: 'cat.system', tabs: ['chat'] },
  { icon: '🖥️', catKey: 'cat.infrastructure', tabs: ['hosts', 'terminal', 'monitor', 'apm', 'clusters'] },
  { icon: '🛡️', catKey: 'cat.security', tabs: ['ids', 'container-sec', 'secrets-scan', 'compliance', 'threats'] },
  { icon: '🤖', catKey: 'cat.automation', tabs: ['change-risk', 'inspection', 'cicd', 'jobs'] },
  { icon: '🔔', catKey: 'cat.monitor', tabs: ['health', 'metrics', 'incidents', 'slos', 'alert-diagnosis', 'rca-analysis'] },
  { icon: '🧠', catKey: 'cat.intelligence', tabs: ['diagnostics', 'reports', 'advisor', 'timeline', 'audit-log-view', 'session-replay'] },
  { icon: '🔧', catKey: 'cat.integration', tabs: ['backup', 'audit', 'users', 'gitops', 'dashboard-layouts'] },
];

/* ── 侧边栏分类名称翻译 ── */
const CAT_KEY_LABELS: Record<string, string> = {
  'cat.overview': '总览大屏',
  'cat.dashboard': '可配置面板',
  'cat.system': '系统管理',
  'cat.infrastructure': '基础设施',
  'cat.security': '安全合规',
  'cat.automation': '自动化',
  'cat.monitor': '监控告警',
  'cat.intelligence': '智能分析',
  'cat.integration': '集成管理',
};

/* ── AppShell ── */
function AppShell({ initialTab }: { initialTab?: Tab } = {}) {
  const [tab, setTab] = useState<Tab>(initialTab ?? 'dashboard');
  const { token, username, logout, role } = useAuthStore();
  const { isUnlocked, checkStatus } = useVaultStore();
  const { isDark, toggleDark } = useTheme();
  const { t, lang, setLang } = useI18n();
  const navigate = useNavigate();
  const location = useLocation();

  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

  useEffect(() => {
    const firstSegment = location.pathname.split('/')[1] as Tab;
    if ((ALL_TABS as readonly string[]).includes(firstSegment)) {
      setTab(firstSegment);
      for (const item of SIDEBAR_ITEMS) {
        if (item.tabs.includes(firstSegment)) {
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
      case 'ops-dashboard': return <OpsDashboardPage />;
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
      case 'terminal': return <TerminalPage />;
      case 'audit': return <AuditLogPage />;
      case 'users': return <UsersPage />;
      case 'alert-rules': return <AlertRulesPage />;
      case 'alert-history': return <AlertHistoryPage />;
      case 'channels': return <NotificationChannelsPage />;
      case 'cmdb': return <CMDBPage />;
      case 'timeline': return <TimelinePage />;
      case 'cicd': return <CICDPage />;
      case 'metrics': return <MetricsVizPage />;
      case 'jobs': return <JobsPage />;
      case 'diagnostics': return <DiagnosticsPage />;
      case 'reports': return <ReportsPage />;
      case 'backup': return <BackupRestorePage />;
      case 'incidents': return <IncidentsPage />;
      case 'vulnerabilities': return <VulnerabilitiesPage />;
      case 'predictions': return <PredictionsPage />;
      case 'slos': return <SLOsPage />;
      case 'soar': return <SOARPage />;
      case 'remediation': return <RemediationPage />;
      case 'secrets-scan': return <SecretsScanPage />;
      case 'compliance': return <CompliancePage />;
      case 'threats': return <ThreatsPage />;
      case 'change-analysis': return <ChangeAnalysisPage />;
      case 'log-intel': return <LogIntelligencePage />;
      case 'oncall': return <OnCallPage />;
      case 'chaos': return <ChaosPage />;
      case 'finops': return <FinOpsPage />;
      case 'apm': return <APMPage />;
      case 'change-risk': return <ChangeRiskPage />;
      case 'inspection': return <InspectionPage />;
      case 'ids': return <IdsPage />;
      case 'container-sec': return <ContainerSecPage />;
      case 'alert-diagnosis': return <AlertDiagnosisPage />;
      case 'gitops': return <GitOpsPage />;
      case 'dashboard-layouts': return <DashboardLayoutsPage />;
      case 'audit-log-view': return <AuditLogViewPage />;
      case 'session-replay': return <SessionReplayPage />;
      case 'rca-analysis': return <RCAnalysisPage />;
      case 'clusters': return <ClusterManagerPage />;
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
                <div className={cn('overflow-hidden transition-all duration-300 ease-in-out', isCollapsed ? 'max-h-0 opacity-0' : 'max-h-[500px] opacity-100')}>
                  <div className="pb-0.5 space-y-0.5">
                    {item.tabs
                      .filter(key => hasRequiredRole(role, TAB_ROLES[key] || ['viewer']))
                      .map(key => (
                        <button
                          key={key}
                          onClick={() => navigateTo(key)}
                          className={cn(
                            'flex items-center gap-3 w-full pl-9 pr-3 py-2 text-sm font-medium transition-all duration-200 rounded-md-lg border-l-[3px]',
                            tab === key
                              ? 'glass-card text-md-on-surface shadow-sm border-l-md-primary bg-md-primary/5'
                              : 'text-md-on-surface-variant hover:glass-card hover:text-md-on-surface hover:border-l-md-primary/50 border-l-transparent',
                          )}
                        >
                          <span className="text-lg">{key === 'vault' ? vaultIcon : ICONS[key]}</span>
                          <span>{t('tab.' + key)}</span>
                        </button>
                      ))}
                  </div>
                </div>
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
          <div className="flex items-center gap-3">
            <GlobalSearch />
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
            <Suspense fallback={<LoadingFallback />}>
              <AnimatePresence mode="wait">
                <motion.div
                  key={tab}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, scale: 0.95 }}
                  transition={{ duration: 0.25 }}
                >
                  {renderContent()}
                </motion.div>
              </AnimatePresence>
            </Suspense>
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
  const { shortcuts, showHelp, setShowHelp } = useNavigationShortcuts();

  useKeyboardShortcuts(shortcuts);

  useEffect(() => {
    installGlobalErrorListener();
  }, []);

  return (
    <ErrorBoundary>
      <ShortcutHelp shortcuts={shortcuts} open={showHelp} onClose={() => setShowHelp(false)} />
      <Routes>
        <Route path="/login" element={token ? <Navigate to="/dashboard" replace /> : <LoginPage />} />
        <Route path="/terminal/:hostId" element={token ? <AppShell initialTab="terminal" /> : <Navigate to="/login" replace />} />
        <Route path="/advisor/chat" element={token ? (
          <div className="flex h-screen bg-md-background">
            <div className="flex-1 flex flex-col min-w-0">
              <header className="h-16 flex items-center px-6 bg-md-surface-container/70 backdrop-blur-xl border-b border-md-outline-variant/50">
                <h1 className="text-title-large font-semibold text-md-on-surface">🤖 AI 运维助手</h1>
              </header>
              <main className="flex-1 overflow-hidden p-4 sm:p-6">
                <AdvisorChatPage />
              </main>
            </div>
          </div>
        ) : <Navigate to="/login" replace />} />
        <Route path="/ops-dashboard" element={token ? <Suspense fallback={<div className="flex items-center justify-center h-screen"><div className="h-8 w-8 border-2 border-md-primary border-t-transparent rounded-full animate-spin" /></div>}><OpsDashboardPage /></Suspense> : <Navigate to="/login" replace />} />
        <Route path="/api-docs" element={token ? <Suspense fallback={<div className="flex items-center justify-center h-screen"><div className="h-8 w-8 border-2 border-md-primary border-t-transparent rounded-full animate-spin" /></div>}><ApiDocsPage /></Suspense> : <Navigate to="/login" replace />} />
        <Route path="/*" element={token ? <AppShell /> : <Navigate to="/login" replace />} />
      </Routes>
    </ErrorBoundary>
  );
}
