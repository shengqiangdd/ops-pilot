import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { TimelineEvent } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

export function TimelinePage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [events, setEvents] = useState<TimelineEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [fromDate, setFromDate] = useState('');
  const [toDate, setToDate] = useState('');
  const [typeFilter, setTypeFilter] = useState('alert,audit,operation');
  const [selectedEvent, setSelectedEvent] = useState<TimelineEvent | null>(null);

  const load = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    setError(null);
    try {
      const params: Record<string, string> = {};
      if (fromDate) params.from = fromDate;
      if (toDate) params.to = toDate;
      if (typeFilter) params.types = typeFilter;
      params.limit = '100';

      const data = await api.getTimelineEvents(token, params);
      setEvents(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load timeline');
    } finally {
      setLoading(false);
    }
  }, [token, fromDate, toDate, typeFilter]);

  useEffect(() => { load(); }, [load]);

  const typeIcon = (type: string) => {
    switch (type) {
      case 'alert': return '🔔';
      case 'audit': return '📋';
      case 'operation': return '⚙️';
      default: return '📌';
    }
  };

  const typeColor = (type: string) => {
    switch (type) {
      case 'alert': return 'bg-red-500';
      case 'audit': return 'bg-blue-500';
      case 'operation': return 'bg-green-500';
      default: return 'bg-md-outline';
    }
  };

  const severityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'text-red-500 bg-red-500/10';
      case 'warning': return 'text-amber-500 bg-amber-500/10';
      case 'info': return 'text-blue-500 bg-blue-500/10';
      case 'success': return 'text-green-500 bg-green-500/10';
      default: return 'text-md-on-surface-variant bg-md-surface-container-high';
    }
  };

  // Group events by date
  const groupedEvents = events.reduce<Record<string, TimelineEvent[]>>((acc, event) => {
    const date = event.timestamp.split('T')[0] || event.timestamp.split(' ')[0] || 'Unknown';
    if (!acc[date]) acc[date] = [];
    acc[date].push(event);
    return acc;
  }, {});


  if (loading) return <LoadingState skeleton="list" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.timeline')}
        </h2>
        <button
          onClick={load}
          disabled={loading}
          className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors"
        >
          {loading ? t('timeline.loading') : t('timeline.reload')}
        </button>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {/* Filters */}
      <div className="glass-card rounded-md-xl p-4">
        <div className="flex flex-wrap items-end gap-3">
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">{t('timeline.filter.from')}</label>
            <input type="datetime-local" value={fromDate} onChange={(e) => setFromDate(e.target.value)}
              className="bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none text-md-on-surface" />
          </div>
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">{t('timeline.filter.to')}</label>
            <input type="datetime-local" value={toDate} onChange={(e) => setToDate(e.target.value)}
              className="bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none text-md-on-surface" />
          </div>
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">{t('timeline.filter.types')}</label>
            <div className="flex gap-2">
              {['alert', 'audit', 'operation'].map((type) => (
                <button
                  key={type}
                  onClick={() => {
                    const types = typeFilter.split(',').filter(Boolean);
                    if (types.includes(type)) {
                      setTypeFilter(types.filter(t => t !== type).join(','));
                    } else {
                      setTypeFilter([...types, type].join(','));
                    }
                  }}
                  className={cn(
                    'px-3 py-1.5 text-sm rounded-md-full transition-colors',
                    typeFilter.includes(type)
                      ? 'bg-md-primary text-md-on-primary'
                      : 'bg-md-surface-container-high text-md-on-surface-variant hover:bg-md-surface-container-highest',
                  )}
                >
                  {typeIcon(type)} {t(`timeline.type.${type}`)}
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Timeline */}
      {loading ? (
        <div className="flex justify-center py-8">
          <div className="h-8 w-8 border-2 border-md-primary border-t-transparent rounded-full animate-spin" />
        </div>
      ) : Object.keys(groupedEvents).length === 0 ? (
        <div className="text-center py-8 text-body-medium text-md-on-surface-variant">
          {t('timeline.empty')}
        </div>
      ) : (
        <div className="space-y-6">
          {Object.entries(groupedEvents).map(([date, dayEvents]) => (
            <div key={date}>
              <div className="flex items-center gap-3 mb-3">
                <div className="text-title-small font-semibold text-md-on-surface">{date}</div>
                <div className="flex-1 h-px bg-md-outline-variant" />
                <span className="text-label-small text-md-on-surface-variant">{dayEvents.length} {t('timeline.events')}</span>
              </div>

              <div className="relative pl-8">
                {/* Timeline line */}
                <div className="absolute left-3 top-0 bottom-0 w-0.5 bg-md-outline-variant" />

                {dayEvents.map((event) => (
                  <div key={event.id} className="relative mb-4 last:mb-0">
                    {/* Timeline dot */}
                    <div className={cn('absolute -left-5 top-3 w-3 h-3 rounded-full border-2 border-white', typeColor(event.type))} />

                    {/* Event card */}
                    <button
                      onClick={() => setSelectedEvent(event)}
                      className="w-full text-left glass-card rounded-md-lg p-3 hover:shadow-md-1 transition-all"
                    >
                      <div className="flex items-start gap-3">
                        <span className="text-lg">{typeIcon(event.type)}</span>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-1">
                            <span className="text-body-medium font-medium text-md-on-surface truncate">{event.title}</span>
                            <span className={cn('text-xs font-medium px-2 py-0.5 rounded-md-sm', severityColor(event.severity))}>
                              {event.severity}
                            </span>
                          </div>
                          <p className="text-body-small text-md-on-surface-variant line-clamp-1">{event.description}</p>
                          <div className="flex items-center gap-3 mt-1 text-label-small text-md-on-surface-variant">
                            <span>{event.timestamp.split('T')[1]?.split('.')[0] || event.timestamp}</span>
                            <span>•</span>
                            <span>{event.source}</span>
                          </div>
                        </div>
                      </div>
                    </button>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Event Detail Modal */}
      {selectedEvent && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" onClick={() => setSelectedEvent(null)}>
          <div className="glass-card rounded-md-2xl p-6 w-full max-w-lg shadow-md-3 animate-scale-in" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-3">
                <span className="text-2xl">{typeIcon(selectedEvent.type)}</span>
                <div>
                  <h3 className="text-title-large font-semibold text-md-on-surface">{selectedEvent.title}</h3>
                  <p className="text-label-medium text-md-on-surface-variant">{selectedEvent.type}</p>
                </div>
              </div>
              <button onClick={() => setSelectedEvent(null)} className="w-8 h-8 rounded-md-full flex items-center justify-center hover:bg-md-surface-container-high transition-colors">
                <svg className="w-5 h-5 text-md-on-surface-variant" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <div className="space-y-4">
              <div className="flex items-center gap-2">
                <span className={cn('text-sm font-medium px-3 py-1 rounded-full', severityColor(selectedEvent.severity))}>
                  {selectedEvent.severity}
                </span>
                <span className="text-sm text-md-on-surface-variant">{selectedEvent.source}</span>
              </div>

              <div className="grid grid-cols-2 gap-3 text-sm">
                <div>
                  <p className="text-label-small text-md-on-surface-variant">{t('timeline.detail.time')}</p>
                  <p className="text-body-medium text-md-on-surface">{selectedEvent.timestamp}</p>
                </div>
                <div>
                  <p className="text-label-small text-md-on-surface-variant">{t('timeline.detail.type')}</p>
                  <p className="text-body-medium text-md-on-surface">{selectedEvent.type}</p>
                </div>
              </div>

              <div>
                <p className="text-label-small text-md-on-surface-variant mb-1">{t('timeline.detail.description')}</p>
                <p className="text-body-medium text-md-on-surface bg-md-surface-container-highest rounded-md-sm p-3">{selectedEvent.description}</p>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
