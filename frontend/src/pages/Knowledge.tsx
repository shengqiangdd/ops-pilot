import { useCallback, useState } from 'react';
import { api } from '../api/client';
import type { KnowledgeEntry } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';

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
    <div className="space-y-6 animate-slide-up">
      <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">Knowledge Base</h2>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}

      <div className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 shadow-md-1">
        <h3 className="mb-3 text-title-medium font-medium text-md-on-surface">Search Knowledge</h3>
        <div className="flex gap-2">
          <input value={query} onChange={(e) => setQuery(e.target.value)} onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            className="flex-1 bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
            placeholder="Search keywords..." />
          <button onClick={handleSearch} disabled={loading || !query}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
            {loading ? 'Searching...' : 'Search'}
          </button>
        </div>
      </div>

      {results.length > 0 && (
        <div className="space-y-3">
          {results.map((r) => (
            <div key={r.id} className="bg-md-surface-container-low rounded-md-lg p-4 shadow-md-1">
              <div className="flex items-start justify-between">
                <div>
                  <h4 className="text-body-large font-medium text-md-on-surface">{r.title}</h4>
                  <p className="mt-1 text-body-medium text-md-on-surface-variant">Incident: {r.incident_id}</p>
                </div>
                <div className="flex gap-1">
                  {r.tags.map((t) => (
                    <span key={t} className="rounded-md-full bg-md-surface-container-high px-2 py-0.5 text-label-medium text-md-on-surface-variant">{t}</span>
                  ))}
                </div>
              </div>
              <div className="mt-3 grid grid-cols-1 gap-2 sm:grid-cols-2">
                <div>
                  <span className="text-label-medium text-md-on-surface-variant">Root Cause:</span>
                  <p className="text-body-medium text-md-on-surface">{r.root_cause}</p>
                </div>
                <div>
                  <span className="text-label-medium text-md-on-surface-variant">Resolution:</span>
                  <p className="text-body-medium text-md-on-surface">{r.resolution}</p>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      <div className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 shadow-md-1">
        <h3 className="mb-3 text-title-medium font-medium text-md-on-surface">Extract Knowledge from Incident</h3>
        <div className="flex gap-2">
          <input value={incidentId} onChange={(e) => setIncidentId(e.target.value)}
            className="flex-1 bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface"
            placeholder="INC-001" />
          <button onClick={handleExtract} disabled={loading || !incidentId}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
            {loading ? 'Extracting...' : 'Extract'}
          </button>
        </div>
      </div>

      {extracted && (
        <div className="bg-md-tertiary-container text-md-on-tertiary-container rounded-md-lg p-4 shadow-md-1">
          <h4 className="text-body-large font-medium">Extracted: {extracted.title}</h4>
          <p className="mt-1 text-body-medium">Root Cause: {extracted.root_cause}</p>
          <p className="text-body-medium">Resolution: {extracted.resolution}</p>
        </div>
      )}
    </div>
  );
}
