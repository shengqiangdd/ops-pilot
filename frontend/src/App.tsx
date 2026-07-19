import { useEffect } from 'react';
import { Routes, Route, Navigate, useNavigate, useLocation } from 'react-router-dom';
import { ModuleBrowser } from './components/ModuleBrowser';
import { HealthDashboard } from './components/HealthDashboard';
import { AgentChat } from './components/AgentChat';
import { HostsPage } from './pages/Hosts';
import { VaultPage } from './pages/Vault';
import { LoginPage } from './pages/Login';
import { useAuthStore } from './stores/useAuthStore';
import { useVaultStore } from './stores/useVaultStore';
import { cn } from './lib/cn';

type Tab = 'chat' | 'modules' | 'hosts' | 'vault' | 'health';

function AppShell() {
  const [tab, setTab] = React.useState<Tab>('modules');
  const { token, username, logout } = useAuthStore();
  const { isUnlocked, checkStatus } = useVaultStore();
  const navigate = useNavigate();
  const location = useLocation();

  // Sync tab from URL path
  useEffect(() => {
    const path = location.pathname.slice(1) as Tab;
    if (['chat', 'modules', 'hosts', 'vault', 'health'].includes(path)) {
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

  return (
    <div className="min-h-screen bg-gray-100">
      <header className="border-b border-gray-200 bg-white">
        <div className="mx-auto flex max-w-7xl items-center gap-6 px-6 py-4">
          <h1 className="text-lg font-bold text-gray-900">OpsPilot</h1>
          <nav className="flex gap-1">
            {([
              ['chat', 'Chat'],
              ['modules', 'Modules'],
              ['hosts', 'Hosts'],
              ['vault', isUnlocked ? 'Vault 🔓' : 'Vault 🔒'],
              ['health', 'Health'],
            ] as const).map(([key, label]) => (
              <button
                key={key}
                onClick={() => navigateTo(key)}
                className={cn(
                  'rounded-md px-3 py-1.5 text-sm font-medium',
                  tab === key
                    ? 'bg-blue-50 text-blue-700'
                    : 'text-gray-600 hover:bg-gray-100',
                )}
              >
                {label}
              </button>
            ))}
          </nav>

          <div className="ml-auto flex items-center gap-3">
            <span className="text-sm text-gray-600">{username}</span>
            <button
              onClick={logout}
              className="rounded-md px-3 py-1.5 text-sm font-medium text-gray-600 hover:bg-gray-100"
            >
              Logout
            </button>
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-7xl px-6 py-8">
        {tab === 'chat' && <AgentChat />}
        {tab === 'modules' && <ModuleBrowser />}
        {tab === 'hosts' && <HostsPage />}
        {tab === 'vault' && <VaultPage />}
        {tab === 'health' && <HealthDashboard />}
      </main>
    </div>
  );
}

// We need React import for the component above
import React from 'react';

export function App() {
  const { token } = useAuthStore();

  return (
    <Routes>
      <Route path="/login" element={token ? <Navigate to="/modules" replace /> : <LoginPage />} />
      <Route path="/*" element={token ? <AppShell /> : <Navigate to="/login" replace />} />
    </Routes>
  );
}
