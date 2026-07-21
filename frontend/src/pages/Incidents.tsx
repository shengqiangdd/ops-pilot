import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { Incident, IncidentDetail, IncidentStats } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

export function IncidentsPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [incidents, setIncidents] = useState<Incident[]>([]);
  const [stats, setStats] = useState<IncidentStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedIncident, setSelectedIncident] = useState<IncidentDetail | null>(null);
  const [statusFilter, setStatusFilter] = useState('');
  const [assignModal, setAssignModal] = useState<string | null>(null);
  const [assignTo, setAssignTo] = useState('');

  const loadIncidents = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const params: Record<string, string> = {};
      if (statusFilter) params.status = statusFilter;
      const data = await api.listIncidents(token, params);
      setIncidents(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load incidents');
    } finally {
      setLoading(false);
    }
  }, [token, statusFilter]);

  const loadStats = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.getIncidentStats(token);
      setStats(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load stats');
    }
  }, [token]);

  useEffect(() => { loadIncidents(); loadStats(); }, [loadIncidents, loadStats]);

  const handleViewIncident = async (id: string) => {
    try {
      const detail = await api.getIncident(token!, id);
      setSelectedIncident(detail);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load incident');
    }
  };

  const handleUpdateStatus = async (id: string, status: string) => {
    try {
      await api.updateIncident(token!, id, { status });
      await loadIncidents();
      await loadStats();
      if (selectedIncident?.incident.id === id) {
        setSelectedIncident(null);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to update incident');
    }
  };

  const handleAssign = async () => {
    if (!assignModal || !assignTo) return;
    try {
      await api.assignIncident(token!, assignModal, { assigned_to: assignTo });
      setAssignModal(null);
      setAssignTo('');
      await loadIncidents();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to assign incident');
    }
  };

  const severityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      case 'high': return 'bg-orange-500/10 text-orange-600 dark:text-orange-400';
      case 'medium': return 'bg-amber-500/10 text-amber-600 dark:text-amber-400';
      case 'low': return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };

  const statusColor = (status: string) => {
    switch (status) {
      case 'open': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      case 'acknowledged': return 'bg-amber-500/10 text-amber-600 dark:text-amber-400';
      case 'resolved': return 'bg-green-500/10 text-green-600 dark:text-green-400';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };


  if (loading) return <LoadingState skeleton="list" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.incidents')}
        </h2>
        <button onClick={loadIncidents} disabled={loading}
          className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors">
          {loading ? t('incidents.loading') : t('incidents.reload')}
        </button>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium flex items-center justify-between">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="text-sm underline">{t('incidents.dismiss')}</button>
        </div>
      )}

      {/* Stats Cards */}
      {stats && (
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-md-primary">{stats.total}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('incidents.stats.total')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-red-500">{stats.open}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('incidents.stats.open')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-amber-500">{stats.acknowledged}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('incidents.stats.acknowledged')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-green-500">{stats.resolved}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('incidents.stats.resolved')}</p>
          </div>
        </div>
      )}

      {/* Filter */}
      <div className="glass-card rounded-md-xl p-4">
        <div className="flex flex-wrap items-center gap-3">
          <label className="text-label-medium text-md-on-surface-variant">{t('incidents.filter.status')}</label>
          <div className="flex gap-2">
            {['', 'open', 'acknowledged', 'resolved'].map((status) => (
              <button
                key={status}
                onClick={() => setStatusFilter(status)}
                className={cn(
                  'px-3 py-1.5 text-sm rounded-md-full transition-colors',
                  statusFilter === status
                    ? 'bg-md-primary text-md-on-primary'
                    : 'bg-md-surface-container-high text-md-on-surface-variant hover:bg-md-surface-container-highest',
                )}
              >
                {status || t('incidents.filter.all')}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Incidents List */}
      <div className="space-y-3">
        {incidents.map((incident) => (
          <div
            key={incident.id}
            className="glass-card rounded-md-xl p-4 cursor-pointer hover:shadow-md-2 transition-all"
            onClick={() => handleViewIncident(incident.id)}
          >
            <div className="flex items-start justify-between mb-2">
              <div className="flex items-center gap-3">
                <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', severityColor(incident.severity))}>
                  {incident.severity}
                </span>
                <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', statusColor(incident.status))}>
                  {incident.status}
                </span>
              </div>
              <span className="text-label-small text-md-on-surface-variant">
                {new Date(incident.last_seen).toLocaleString()}
              </span>
            </div>
            <h3 className="text-body-medium font-medium text-md-on-surface mb-1">{incident.name}</h3>
            <p className="text-body-small text-md-on-surface-variant line-clamp-2">{incident.summary}</p>
            <div className="flex items-center gap-4 mt-2 text-label-small text-md-on-surface-variant">
              <span>🖥️ {incident.host_id || t('incidents.unknown_host')}</span>
              <span>🔔 {incident.alert_count} {t('incidents.alerts')}</span>
              {incident.assigned_to && <span>👤 {incident.assigned_to}</span>}
            </div>
          </div>
        ))}
        {!loading && incidents.length === 0 && (
          <div className="glass-card rounded-md-xl p-8 text-center">
            <div className="text-4xl mb-3">✅</div>
            <p className="text-body-medium text-md-on-surface-variant">{t('incidents.no_incidents')}</p>
          </div>
        )}
      </div>

      {/* Incident Detail Modal */}
      {selectedIncident && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" onClick={() => setSelectedIncident(null)}>
          <div className="glass-card rounded-md-2xl p-6 w-full max-w-2xl max-h-[80vh] overflow-auto shadow-md-3 animate-scale-in" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-title-large font-semibold text-md-on-surface">{selectedIncident.incident.name}</h3>
              <button onClick={() => setSelectedIncident(null)} className="w-8 h-8 rounded-md-full flex items-center justify-center hover:bg-md-surface-container-high transition-colors">
                <svg className="w-5 h-5 text-md-on-surface-variant" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <div className="flex items-center gap-3 mb-4">
              <span className={cn('text-sm font-medium px-3 py-1 rounded-full', severityColor(selectedIncident.incident.severity))}>{selectedIncident.incident.severity}</span>
              <span className={cn('text-sm font-medium px-3 py-1 rounded-full', statusColor(selectedIncident.incident.status))}>{selectedIncident.incident.status}</span>
            </div>

            <p className="text-body-medium text-md-on-surface-variant mb-4">{selectedIncident.incident.summary}</p>

            {/* Actions */}
            <div className="flex gap-2 mb-4">
              {selectedIncident.incident.status === 'open' && (
                <button onClick={() => handleUpdateStatus(selectedIncident.incident.id, 'acknowledged')}
                  className="px-3 py-1.5 text-sm rounded-md-full bg-amber-500/10 text-amber-600 hover:bg-amber-500/20 transition-colors">
                  {t('incidents.acknowledge')}
                </button>
              )}
              {selectedIncident.incident.status !== 'resolved' && (
                <button onClick={() => handleUpdateStatus(selectedIncident.incident.id, 'resolved')}
                  className="px-3 py-1.5 text-sm rounded-md-full bg-green-500/10 text-green-600 hover:bg-green-500/20 transition-colors">
                  {t('incidents.resolve')}
                </button>
              )}
              <button onClick={() => { setAssignModal(selectedIncident.incident.id); }}
                className="px-3 py-1.5 text-sm rounded-md-full bg-md-primary/10 text-md-primary hover:bg-md-primary/20 transition-colors">
                {t('incidents.assign')}
              </button>
            </div>

            {/* Alerts Timeline */}
            <h4 className="text-title-small font-semibold text-md-on-surface mb-2">{t('incidents.associated_alerts')}</h4>
            <div className="space-y-2">
              {selectedIncident.alerts.map((alert) => (
                <div key={alert.id} className="flex items-center gap-3 px-3 py-2 rounded-md-lg bg-md-surface-container-highest/50">
                  <span className={cn('h-2 w-2 rounded-full', alert.alert_severity === 'critical' ? 'bg-red-500' : alert.alert_severity === 'warning' ? 'bg-amber-500' : 'bg-blue-500')} />
                  <span className="flex-1 text-body-small text-md-on-surface">{alert.alert_message}</span>
                  <span className="text-label-small text-md-on-surface-variant">{new Date(alert.triggered_at).toLocaleTimeString()}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Assign Modal */}
      {assignModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" onClick={() => setAssignModal(null)}>
          <div className="glass-card rounded-md-2xl p-6 w-full max-w-sm shadow-md-3 animate-scale-in" onClick={e => e.stopPropagation()}>
            <h3 className="text-title-large font-semibold text-md-on-surface mb-4">{t('incidents.assign_to')}</h3>
            <input
              type="text"
              value={assignTo}
              onChange={(e) => setAssignTo(e.target.value)}
              className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface mb-4"
              placeholder={t('incidents.assign_placeholder')}
            />
            <div className="flex justify-end gap-2">
              <button onClick={() => setAssignModal(null)}
                className="px-4 py-2 text-sm rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">
                {t('incidents.cancel')}
              </button>
              <button onClick={handleAssign}
                className="px-4 py-2 text-sm rounded-md-lg bg-md-primary text-md-on-primary hover:shadow-md-2 transition-all">
                {t('incidents.assign_btn')}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
