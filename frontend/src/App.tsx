import { useState } from 'react';
import { ModuleBrowser } from './components/ModuleBrowser';
import { HealthDashboard } from './components/HealthDashboard';
import { AgentChat } from './components/AgentChat';
import { HostsPage } from './pages/Hosts';
import { useAuthStore } from './stores/useAuthStore';
import { api } from './api/client';
import { cn } from './lib/cn';

type Tab = 'chat' | 'modules' | 'hosts' | 'health';

export function App() {
  const [tab, setTab] = useState<Tab>('modules');
  const { token, username, setAuth, logout } = useAuthStore();
  const [loginOpen, setLoginOpen] = useState(false);
  const [loginUser, setLoginUser] = useState('');
  const [loginPass, setLoginPass] = useState('');
  const [loginError, setLoginError] = useState<string | null>(null);
  const [loggingIn, setLoggingIn] = useState(false);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoggingIn(true);
    setLoginError(null);
    try {
      const resp = await api.login(loginUser, loginPass);
      setAuth(resp.token, loginUser);
      setLoginOpen(false);
      setLoginUser('');
      setLoginPass('');
    } catch (err) {
      setLoginError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setLoggingIn(false);
    }
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
              ['health', 'Health'],
            ] as const).map(([key, label]) => (
              <button
                key={key}
                onClick={() => setTab(key)}
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

          <div className="ml-auto">
            {token ? (
              <div className="flex items-center gap-3">
                <span className="text-sm text-gray-600">{username}</span>
                <button
                  onClick={logout}
                  className="rounded-md px-3 py-1.5 text-sm font-medium text-gray-600 hover:bg-gray-100"
                >
                  Logout
                </button>
              </div>
            ) : (
              <button
                onClick={() => setLoginOpen(!loginOpen)}
                className={cn(
                  'rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white',
                  'hover:bg-blue-700',
                )}
              >
                Login
              </button>
            )}
          </div>
        </div>

        {loginOpen && !token && (
          <div className="border-t border-gray-200 bg-gray-50 px-6 py-4">
            <form onSubmit={handleLogin} className="flex items-center gap-3">
              <input
                type="text"
                placeholder="Username"
                value={loginUser}
                onChange={(e) => setLoginUser(e.target.value)}
                className="rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              />
              <input
                type="password"
                placeholder="Password"
                value={loginPass}
                onChange={(e) => setLoginPass(e.target.value)}
                className="rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              />
              <button
                type="submit"
                disabled={loggingIn}
                className="rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
              >
                {loggingIn ? 'Logging in...' : 'Sign In'}
              </button>
              {loginError && (
                <span className="text-sm text-red-600">{loginError}</span>
              )}
            </form>
          </div>
        )}
      </header>

      <main className="mx-auto max-w-7xl px-6 py-8">
        {tab === 'chat' && <AgentChat />}
        {tab === 'modules' && <ModuleBrowser />}
        {tab === 'hosts' && <HostsPage />}
        {tab === 'health' && <HealthDashboard />}
      </main>
    </div>
  );
}
