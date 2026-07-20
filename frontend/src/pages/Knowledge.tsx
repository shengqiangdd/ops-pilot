import { useCallback, useState } from 'react';
import { api } from '../api/client';
import type { KnowledgeEntry } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

export function KnowledgePage() {
  const { token } = useAuthStore();
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<KnowledgeEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [incidentId, setIncidentId] = useState('');
  const [extracted, setExtracted] = useState<KnowledgeEntry | null>(null);

  const handleSearch = useCallback(async () => {
    if (!token || !query) return;
    setLoading(true);
    setError(null);
    try {
      const res = await api.searchKnowledge(token, query);
      setResults(res.results);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setLoading(false);
    }
  }, [token, query]);

  const handleExtract = useCallback(async () => {
    if (!token || !incidentId) return;
    setLoading(true);
    setError(null);
    try {
      const entry = await api.extractKnowledge(token, incidentId);
      setExtracted(entry);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setLoading(false);
    }
  }, [token, incidentId]);

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold text-gray-900">运维知识库</h2>

      {error && <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>}

      {/* Search */}
      <div className="rounded-lg border border-gray-200 bg-white p-5 shadow-sm">
        <h3 className="mb-3 text-base font-semibold text-gray-900">搜索知识库</h3>
        <div className="flex gap-2">
          <input value={query} onChange={(e) => setQuery(e.target.value)} onKeyDown={(e) => e.key === 'Enter' && handleSearch()} className="flex-1 rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500" placeholder="搜索关键词..." />
          <button onClick={handleSearch} disabled={loading || !query} className={cn('rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50')}>
            {loading ? '搜索中...' : '搜索'}
          </button>
        </div>
      </div>

      {/* Search Results */}
      {results.length > 0 && (
        <div className="space-y-3">
          {results.map((r) => (
            <div key={r.id} className="rounded-lg border border-gray-200 bg-white p-4 shadow-sm">
              <div className="flex items-start justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-gray-900">{r.title}</h4>
                  <p className="mt-1 text-xs text-gray-500">事件: {r.incident_id}</p>
                </div>
                <div className="flex gap-1">
                  {r.tags.map((t) => (
                    <span key={t} className="rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-600">{t}</span>
                  ))}
                </div>
              </div>
              <div className="mt-3 grid grid-cols-1 gap-2 sm:grid-cols-2">
                <div>
                  <span className="text-xs font-medium text-gray-500">根因:</span>
                  <p className="text-sm text-gray-700">{r.root_cause}</p>
                </div>
                <div>
                  <span className="text-xs font-medium text-gray-500">解决方案:</span>
                  <p className="text-sm text-gray-700">{r.resolution}</p>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Extract Knowledge */}
      <div className="rounded-lg border border-gray-200 bg-white p-5 shadow-sm">
        <h3 className="mb-3 text-base font-semibold text-gray-900">从事件中提取知识</h3>
        <div className="flex gap-2">
          <input value={incidentId} onChange={(e) => setIncidentId(e.target.value)} className="flex-1 rounded-md border border-gray-300 px-3 py-1.5 text-sm" placeholder="INC-001" />
          <button onClick={handleExtract} disabled={loading || !incidentId} className={cn('rounded-md bg-green-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-green-700 disabled:opacity-50')}>
            {loading ? '提取中...' : '提取知识'}
          </button>
        </div>
      </div>

      {extracted && (
        <div className="rounded-lg border border-blue-200 bg-blue-50 p-4 shadow-sm">
          <h4 className="text-sm font-semibold text-blue-900">已提取: {extracted.title}</h4>
          <p className="mt-1 text-sm text-blue-700">根因: {extracted.root_cause}</p>
          <p className="text-sm text-blue-700">解决方案: {extracted.resolution}</p>
        </div>
      )}
    </div>
  );
}
