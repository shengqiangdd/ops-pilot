import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { ChaosExperiment, ChaosStats, CreateChaosExperimentInput } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

const EMPTY_FORM: CreateChaosExperimentInput = {
  name: '',
  description: '',
  fault_type: 'cpu_storm',
  duration_seconds: 60,
  target_type: 'host',
  params_json: '{}',
};

export function ChaosPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [experiments, setExperiments] = useState<ChaosExperiment[]>([]);
  const [stats, setStats] = useState<ChaosStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<CreateChaosExperimentInput>(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [runningId, setRunningId] = useState<string | null>(null);

  const loadAll = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const [exp, st] = await Promise.all([
        api.listChaosExperiments(token),
        api.getChaosStats(token),
      ]);
      setExperiments(exp);
      setStats(st);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load data');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { loadAll(); }, [loadAll]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    try {
      await api.createChaosExperiment(token!, form);
      setForm(EMPTY_FORM);
      setShowForm(false);
      await loadAll();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create');
    } finally {
      setSubmitting(false);
    }
  };

  const handleRun = async (id: string) => {
    setRunningId(id);
    try {
      await api.runChaosExperiment(token!, id);
      await loadAll();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to run');
    } finally {
      setRunningId(null);
    }
  };

  const handleStop = async (id: string) => {
    try {
      await api.stopChaosExperiment(token!, id);
      await loadAll();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to stop');
    }
  };

  const handleDelete = async (id: string) => {
    if (!window.confirm(t('chaos.delete_confirm'))) return;
    try {
      await api.deleteChaosExperiment(token!, id);
      await loadAll();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete');
    }
  };

  const statusColor = (status: string) => {
    switch (status) {
      case 'running': return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
      case 'completed': return 'bg-green-500/10 text-green-600 dark:text-green-400';
      case 'failed': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      case 'stopped': return 'bg-md-surface-container-high text-md-on-surface-variant';
      default: return 'bg-amber-500/10 text-amber-600 dark:text-amber-400';
    }
  };

  const faultIcon = (type: string) => {
    const icons: Record<string, string> = {
      cpu_storm: '🔥', memory_pressure: '💾', disk_fill: '💿',
      network_latency: '🌐', network_loss: '📡', process_kill: '💀', dns_failure: '🔗',
    };
    return icons[type] || '⚡';
  };


  if (loading) return <LoadingState skeleton="list" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.chaos')}
        </h2>
        <div className="flex gap-2">
          <button onClick={loadAll} disabled={loading} className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors">
            {loading ? t('chaos.loading') : t('chaos.reload')}
          </button>
          <button onClick={() => setShowForm(!showForm)} className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all">
            {showForm ? t('chaos.cancel') : t('chaos.add')}
          </button>
        </div>
      </div>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}

      {/* Stats */}
      {stats && (
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
          <div className="glass-card rounded-md-xl p-4 text-center"><p className="text-headline-medium font-bold text-md-primary">{stats.total_experiments}</p><p className="text-label-small text-md-on-surface-variant">{t('chaos.stats.total')}</p></div>
          <div className="glass-card rounded-md-xl p-4 text-center"><p className="text-headline-medium font-bold text-green-500">{stats.completed}</p><p className="text-label-small text-md-on-surface-variant">{t('chaos.stats.completed')}</p></div>
          <div className="glass-card rounded-md-xl p-4 text-center"><p className="text-headline-medium font-bold text-blue-500">{stats.running}</p><p className="text-label-small text-md-on-surface-variant">{t('chaos.stats.running')}</p></div>
          <div className="glass-card rounded-md-xl p-4 text-center"><p className="text-headline-medium font-bold text-red-500">{stats.failed}</p><p className="text-label-small text-md-on-surface-variant">{t('chaos.stats.failed')}</p></div>
        </div>
      )}

      {/* Create Form */}
      {showForm && (
        <form onSubmit={handleCreate} className="bg-md-surface-container-low rounded-md-lg p-4 space-y-3 animate-slide-up">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('chaos.name')}</label>
              <input type="text" required value={form.name} onChange={e => setForm({ ...form, name: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('chaos.fault_type')}</label>
              <select value={form.fault_type} onChange={e => setForm({ ...form, fault_type: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface">
                {['cpu_storm', 'memory_pressure', 'disk_fill', 'network_latency', 'network_loss', 'process_kill', 'dns_failure'].map(ft => (
                  <option key={ft} value={ft}>{ft.replace(/_/g, ' ')}</option>
                ))}
              </select>
            </div>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('chaos.duration')}</label>
              <input type="number" min={5} max={3600} value={form.duration_seconds} onChange={e => setForm({ ...form, duration_seconds: Number(e.target.value) })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('chaos.target_type')}</label>
              <select value={form.target_type} onChange={e => setForm({ ...form, target_type: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface">
                <option value="host">{t('chaos.target_host')}</option>
                <option value="service">{t('chaos.target_service')}</option>
              </select>
            </div>
          </div>
          <div>
            <label className="block text-label-large text-md-on-surface mb-1">{t('chaos.description')}</label>
            <input type="text" value={form.description} onChange={e => setForm({ ...form, description: e.target.value })}
              className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
          </div>
          <div className="flex justify-end gap-2">
            <button type="button" onClick={() => setShowForm(false)} className="px-4 py-2 text-sm rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">{t('chaos.cancel')}</button>
            <button type="submit" disabled={submitting} className="px-4 py-2 text-sm rounded-md-lg bg-md-primary text-md-on-primary hover:shadow-md-2 disabled:opacity-50 transition-all">{submitting ? t('chaos.creating') : t('chaos.create')}</button>
          </div>
        </form>
      )}

      {/* Experiments Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {experiments.map(exp => (
          <div key={exp.id} className="glass-card rounded-md-xl p-4">
            <div className="flex items-start justify-between mb-2">
              <div className="flex items-center gap-2">
                <span className="text-xl">{faultIcon(exp.fault_type)}</span>
                <span className="text-body-medium font-medium text-md-on-surface">{exp.name}</span>
              </div>
              <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', statusColor(exp.status))}>{exp.status}</span>
            </div>
            <p className="text-body-small text-md-on-surface-variant mb-2 line-clamp-2">{exp.description || t('chaos.no_description')}</p>
            <div className="flex items-center gap-4 text-label-small text-md-on-surface-variant mb-3">
              <span>⏱️ {exp.duration_seconds}s</span>
              <span>🎯 {exp.target_type}</span>
            </div>
            <div className="flex gap-2">
              {exp.status === 'pending' && (
                <button onClick={() => handleRun(exp.id)} disabled={runningId === exp.id}
                  className="flex-1 text-xs px-3 py-1.5 rounded-md-full bg-green-500/10 text-green-600 hover:bg-green-500/20 transition-colors disabled:opacity-50">
                  {runningId === exp.id ? t('chaos.starting') : t('chaos.run')}
                </button>
              )}
              {exp.status === 'running' && (
                <button onClick={() => handleStop(exp.id)}
                  className="flex-1 text-xs px-3 py-1.5 rounded-md-full bg-red-500/10 text-red-600 hover:bg-red-500/20 transition-colors">
                  {t('chaos.stop')}
                </button>
              )}
              <button onClick={() => handleDelete(exp.id)} className="text-xs px-2 py-1 rounded-md-sm text-md-error hover:bg-md-error-container/30 transition-colors">{t('chaos.delete')}</button>
            </div>
          </div>
        ))}
        {!loading && experiments.length === 0 && <div className="col-span-full glass-card rounded-md-xl p-8 text-center"><p className="text-body-medium text-md-on-surface-variant">{t('chaos.no_experiments')}</p></div>}
      </div>
    </div>
  );
}
