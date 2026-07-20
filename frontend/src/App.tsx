import React, { useEffect } from 'react';
import { Routes, Route, Navigate, useNavigate, useLocation } from 'react-router-dom';
import { ModuleBrowser } from './components/ModuleBrowser';
import { HealthDashboard } from './components/HealthDashboard';
import { AgentChat } from './components/AgentChat';
import { HostsPage } from './pages/Hosts';
import { VaultPage } from './pages/Vault';
import { LoginPage } from './pages/Login';
import { SecurityPage } from './pages/Security';
import { useAuthStore } from './stores/useAuthStore';
import { useVaultStore } from './stores/useVaultStore';
import { cn } from './lib/cn';
import { ErrorBoundary } from './components/ErrorBoundary';

type Tab = 'chat' | 'modules' | 'hosts' | 'vault' | 'security' | 'health';

const ALL_TABS: Tab[] = ['chat', 'modules', 'hosts', 'vault', 'security', 'health'];

const TAB_LABELS: Record<Tab, string> = {
  chat: '💬 Chat',
  modules: '🧩 Modules',
  hosts: '🖥️ Hosts',
  vault: '🔑 Vault',
  security: '🛡️ Security',
  health: '❤️ Health',
};

function AppShell() {
  const [tab, setTab] = React.useState<Tab>('modules');
  const { token, username, logout } = useAuthStore();
  const { isUnlocked, checkStatus } = useVaultStore();
  const navigate = useNavigate();
  const location = useLocation();

  // Sync tab from URL path
  useEffect(() => {
    const path = location.pathname.slice(1) as Tab;
    if ((ALL_TABS as readonly string[]).includes(path)) {
      setTab(path);
    }
  }, [location.pathname]);

  // Check vault status when token changes
  useEffect(() => {
    if (token) {
      checkStatus();
    }
  }, [token, checkStatus]);

  const navigateTo = (key: Tab) => {
    setTab(key);
    navigate('/' + key);
  };

  const vaultLabel = isUnlocked ? '🔓 Vault' : '🔒 Vault';

  return (
    <div className="min-h-screen bg-gray-100">
      <header className="border-b border-gray-200 bg-white">
        <div className="mx-auto flex max-w-7xl items-center gap-4 px-4 py-3 sm:px-6 sm:py-4">
          <h1 className="text-lg font-bold text-gray-900">OpsPilot</h1>

          {/* Desktop Nav */}
          <nav className="hidden gap-1 md:flex">
            {(ALL_TABS as Tab[]).map((key) => (
              <button
                key={key}
                onClick={() => navigateTo(key)}
                className={cn(
                  'rounded-md px-3 py-1.5 text-sm font-medium transition-colors',
                  tab === key
                    ? 'bg-blue-50 text-blue-700'
                    : 'text-gray-600 hover:bg-gray-100',
                )}
              >
                {key === 'vault' ? vaultLabel : TAB_LABELS[key]}
              </button>
            ))}
          </nav>

          {/* Mobile Nav */}
          <nav className="flex gap-1 md:hidden">
            <select
              value={tab}
              onChange={(e) => navigateTo(e.target.value as Tab)}
              className="block w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            >
              {(ALL_TABS as Tab[]).map((key) => (
                <option key={key} value={key}>
                  {key === 'vault' ? vaultLabel : TAB_LABELS[key]}
                </option>
              ))}
            </select>
          </nav>

          <div className="ml-auto flex items-center gap-3">
            <span className="hidden text-sm text-gray-600 sm:inline">{username}</span>
            <button
              onClick={logout}
              className="rounded-md px-3 py-1.5 text-sm font-medium text-gray-600 hover:bg-gray-100"
            >
              Logout
            </button>
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-7xl px-4 py-6 sm:px-6 sm:py-8">
        <ErrorBoundary key={tab}>
          {tab === 'chat' && <AgentChat />}
          {tab === 'modules' && <ModuleBrowser />}
          {tab === 'hosts' && <HostsPage />}
          {tab === 'vault' && <VaultPage />}
          {tab === 'security' && <SecurityPage />}
          {tab === 'health' && <HealthDashboard />}
        </ErrorBoundary>
      </main>
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
