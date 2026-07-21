import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { ChangeEvent, ChangeStats } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

export function ChangeAnalysisPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [events, setEvents] = useState<ChangeEvent[]>([]);
  const [stats, setStats] = useState<ChangeStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedEvent, setSelectedEvent] = useState<ChangeEvent | null>(null);
  const [statusFilter, setStatusFilter] = useState('');

  const loadEvents = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const params: Record<string, string> = {};
      if (statusFilter) params.status = statusFilter;
      const data = await api.listChangeEvents(token, params);
      setEvents(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load events');
    } finally {
      setLoading(false);
    }
  }, [token, statusFilter]);

  const loadStats = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.getChangeStats(token);
      setStats(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load stats');
    }
  }, [token]);

  useEffect(() => { loadEvents(); loadStats(); }, [loadEvents, loadStats]);

  const handleReview = async (eventId: string, status: string) => {
    try {
      await api.reviewChangeEvent(token!, eventId, { status, reviewed_by: 'admin' });
      setSelectedEvent(null);
      await loadEvents();
      await loadStats();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to review');
    }
  };

  const riskColor = (score: number) => {
    if (score > 0.7) return 'text-red-500 bg-red-500/10';
    if (score > 0.4) return 'text-amber-500 bg-amber-500/10';
    return 'text-green-500 bg-green-500/10';
  };

  const statusColor = (status: string) => {
    switch (status) {
      case 'approved': return 'bg-green-500/10 text-green-600 dark:text-green-400';
      case 'rejected': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      case 'pending_review': return 'bg-amber-500/10 text-amber-600 dark:text-amber-400';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };


  if (loading) return <LoadingState skeleton="list" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.change_analysis')}
        </h2>
        <button onClick={loadEvents} disabled={loading}
          className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors">
          {loading ? t('change.loading') : t('change.reload')}
        </button>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {/* Stats */}
      {stats && (
        <div className="grid grid-cols-2 sm:grid-cols-5 gap-4">
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-md-primary">{stats.total}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('change.stats.total')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-amber-500">{stats.pending}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('change.stats.pending')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-green-500">{stats.approved}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('change.stats.approved')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-red-500">{stats.rejected}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('change.stats.rejected')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-red-500">{stats.high_risk}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('change.stats.high_risk')}</p>
          </div>
        </div>
      )}

      {/* Filters */}
      <div className="glass-card rounded-md-xl p-4">
        <div className="flex flex-wrap items-center gap-3">
          <label className="text-label-medium text-md-on-surface-variant">{t('change.filter.status')}</label>
          <div className="flex gap-2">
            {['', 'pending_review', 'approved', 'rejected'].map((status) => (
              <button key={status} onClick={() => setStatusFilter(status)}
                className={cn('px-3 py-1.5 text-sm rounded-md-full transition-colors',
                  statusFilter === status ? 'bg-md-primary text-md-on-primary' : 'bg-md-surface-container-high text-md-on-surface-variant')}>
                {status ? t(`change.status.${status}`) : t('change.filter.all')}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Events Table */}
      <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
        <div className="overflow-x-auto">
          <table className="min-w-full">
            <thead>
              <tr className="border-b border-md-outline-variant">
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('change.col.type')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('change.col.description')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('change.col.source')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('change.col.risk')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('change.col.status')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('change.col.time')}</th>
              </tr>
            </thead>
            <tbody>
              {events.map((event) => (
                <tr key={event.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors cursor-pointer"
                    onClick={() => setSelectedEvent(event)}>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{event.change_type}</td>
                  <td className="px-4 py-3 text-body-small text-md-on-surface-variant max-w-xs truncate">{event.description}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{event.source}</td>
                  <td className="whitespace-nowrap px-4 py-3">
                    <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', riskColor(event.risk_score))}>
                      {(event.risk_score * 100).toFixed(0)}%
                    </span>
                  </td>
                  <td className="whitespace-nowrap px-4 py-3">
                    <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', statusColor(event.status))}>
                      {t(`change.status.${event.status}`)}
                    </span>
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">
                    {new Date(event.created_at).toLocaleDateString()}
                  </td>
                </tr>
              ))}
              {!loading && events.length === 0 && (
                <tr><td colSpan={6} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('change.empty')}</td></tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Detail Modal */}
      {selectedEvent && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" onClick={() => setSelectedEvent(null)}>
          <div className="glass-card rounded-md-2xl p-6 w-full max-w-lg max-h-[80vh] overflow-auto shadow-md-3 animate-scale-in" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-title-large font-semibold text-md-on-surface">{t('change.detail')}</h3>
              <button onClick={() => setSelectedEvent(null)} className="w-8 h-8 rounded-md-full flex items-center justify-center hover:bg-md-surface-container-high transition-colors">
                <svg className="w-5 h-5 text-md-on-surface-variant" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <div className="space-y-4">
              <div className="flex items-center gap-3">
                <span className={cn('text-sm font-medium px-3 py-1 rounded-full', riskColor(selectedEvent.risk_score))}>
                  Risk: {(selectedEvent.risk_score * 100).toFixed(0)}%
                </span>
                <span className={cn('text-sm font-medium px-3 py-1 rounded-full', statusColor(selectedEvent.status))}>
                  {t(`change.status.${selectedEvent.status}`)}
                </span>
              </div>

              <div>
                <p className="text-label-small text-md-on-surface-variant">{t('change.col.description')}</p>
                <p className="text-body-medium text-md-on-surface">{selectedEvent.description}</p>
              </div>

              {selectedEvent.content_diff && (
                <div>
                  <p className="text-label-small text-md-on-surface-variant">{t('change.diff')}</p>
                  <pre className="bg-md-surface-container-highest rounded-md-sm p-3 text-body-small font-mono overflow-auto max-h-40">{selectedEvent.content_diff}</pre>
                </div>
              )}

              {selectedEvent.status === 'pending_review' && (
                <div className="flex gap-2 pt-2">
                  <button onClick={() => handleReview(selectedEvent.id, 'approved')}
                    className="px-4 py-2 text-sm rounded-md-lg bg-green-500/10 text-green-600 hover:bg-green-500/20 transition-colors">
                    {t('change.approve')}
                  </button>
                  <button onClick={() => handleReview(selectedEvent.id, 'rejected')}
                    className="px-4 py-2 text-sm rounded-md-lg bg-red-500/10 text-red-600 hover:bg-red-500/20 transition-colors">
                    {t('change.reject')}
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
