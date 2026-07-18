import { useState } from 'react';
import { ModuleBrowser } from './components/ModuleBrowser';
import { HealthDashboard } from './components/HealthDashboard';
import { cn } from './lib/cn';

type Tab = 'modules' | 'health';

export function App() {
  const [tab, setTab] = useState<Tab>('modules');

  return (
    <div className="min-h-screen bg-gray-100">
      <header className="border-b border-gray-200 bg-white">
        <div className="mx-auto flex max-w-7xl items-center gap-6 px-6 py-4">
          <h1 className="text-lg font-bold text-gray-900">OpsPilot</h1>
          <nav className="flex gap-1">
            {([
              ['modules', 'Modules'],
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
        </div>
      </header>

      <main className="mx-auto max-w-7xl px-6 py-8">
        {tab === 'modules' && <ModuleBrowser />}
        {tab === 'health' && <HealthDashboard />}
      </main>
    </div>
  );
}
