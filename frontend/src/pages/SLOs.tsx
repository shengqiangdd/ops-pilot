import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { SLO, BurnRateAlert, CreateSloInput } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

const EMPTY_FORM: CreateSloInput = {
  name: '',
  description: '',
  service_id: '',
  sli_type: 'availability',
  target_percentage: 99.9,
  window_days: 30,
};

export function SLOsPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [slos, setSlos] = useState<SLO[]>([]);
  const [burnRates, setBurnRates] = useState<BurnRateAlert[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<CreateSloInput>(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);

  const loadSlos = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const data = await api.listSlos(token);
      setSlos(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load SLOs');
    } finally {
      setLoading(false);
    }
  }, [token]);

  const loadBurnRates = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.getBurnRateAlerts(token);
      setBurnRates(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load burn rates');
    }
  }, [token]);

  useEffect(() => { loadSlos(); loadBurnRates(); }, [loadSlos, loadBurnRates]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    try {
      await api.createSlo(token!, form);
      setForm(EMPTY_FORM);
      setShowForm(false);
      await loadSlos();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create SLO');
    } finally {
      setSubmitting(false);
    }
  };

  const handleEvaluate = async (sloId: string) => {
    try {
      await api.evaluateSlo(token!, sloId);
      await loadSlos();
      await loadBurnRates();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to evaluate');
    }
  };

  const statusColor = (status: string) => {
    switch (status) {
      case 'compliant': return 'bg-green-500/10 text-green-600 dark:text-green-400';
      case 'at_risk': return 'bg-amber-500/10 text-amber-600 dark:text-amber-400';
      case 'breached': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };


  if (loading) return <LoadingState skeleton="chart" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.slos')}
        </h2>
        <div className="flex gap-2">
          <button onClick={loadSlos} disabled={loading}
            className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors">
            {loading ? t('slos.loading') : t('slos.reload')}
          </button>
          <button onClick={() => setShowForm(!showForm)}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all">
            {showForm ? t('slos.cancel') : t('slos.add')}
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {/* Create Form */}
      {showForm && (
        <form onSubmit={handleSubmit} className="bg-md-surface-container-low rounded-md-lg p-4 space-y-3 animate-slide-up">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('slos.name')}</label>
              <input type="text" required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('slos.sli_type')}</label>
              <select value={form.sli_type} onChange={(e) => setForm({ ...form, sli_type: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface">
                <option value="availability">{t('slos.type_availability')}</option>
                <option value="latency">{t('slos.type_latency')}</option>
                <option value="error_rate">{t('slos.type_error_rate')}</option>
                <option value="custom">{t('slos.type_custom')}</option>
              </select>
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('slos.target')}</label>
              <input type="number" step="0.1" min="0" max="100" value={form.target_percentage} onChange={(e) => setForm({ ...form, target_percentage: Number(e.target.value) })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('slos.window_days')}</label>
              <input type="number" min="1" max="90" value={form.window_days} onChange={(e) => setForm({ ...form, window_days: Number(e.target.value) })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
          </div>
          <div>
            <label className="block text-label-large text-md-on-surface mb-1">{t('slos.description')}</label>
            <input type="text" value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })}
              className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
          </div>
          <div className="flex justify-end gap-2">
            <button type="button" onClick={() => setShowForm(false)}
              className="px-4 py-2 text-sm rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">
              {t('slos.cancel')}
            </button>
            <button type="submit" disabled={submitting}
              className="px-4 py-2 text-sm rounded-md-lg bg-md-primary text-md-on-primary hover:shadow-md-2 disabled:opacity-50 transition-all">
              {submitting ? t('slos.creating') : t('slos.create')}
            </button>
          </div>
        </form>
      )}

      {/* Burn Rate Alerts */}
      {burnRates.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-title-medium font-semibold text-md-on-surface">{t('slos.burn_rate_alerts')}</h3>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {burnRates.map((alert) => (
              <div key={alert.slo_id} className={cn('glass-card rounded-md-xl p-4 border', statusColor(alert.severity === 'critical' ? 'breached' : 'at_risk'))}>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-body-medium font-medium text-md-on-surface">{alert.slo_name}</span>
                  <span className="text-xs font-medium px-2 py-0.5 rounded-full bg-md-surface-container-highest">{alert.burn_rate.toFixed(1)}%</span>
                </div>
                <div className="grid grid-cols-2 gap-2 text-sm mb-2">
                  <div>
                    <p className="text-label-small text-md-on-surface-variant">{t('slos.budget_remaining')}</p>
                    <p className="text-body-medium font-medium">{alert.error_budget_remaining.toFixed(2)}%</p>
                  </div>
                  <div>
                    <p className="text-label-small text-md-on-surface-variant">{t('slos.time_to_breach')}</p>
                    <p className="text-body-medium font-medium">{alert.estimated_breach_hours.toFixed(1)}h</p>
                  </div>
                </div>
                <p className="text-body-small text-md-on-surface-variant">💡 {alert.suggestion}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* SLO Cards */}
      <div className="space-y-3">
        <h3 className="text-title-medium font-semibold text-md-on-surface">{t('slos.list')}</h3>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {slos.map((slo) => {
            const budgetPercent = slo.target_percentage > 0
              ? Math.max(0, Math.min(100, slo.error_budget_remaining / (100 - slo.target_percentage) * 100))
              : 100;

            return (
              <div key={slo.id} className="glass-card rounded-md-xl p-4">
                <div className="flex items-center justify-between mb-3">
                  <span className="text-body-medium font-medium text-md-on-surface">{slo.name}</span>
                  <span className={cn('text-xs font-medium px-2 py-0.5 rounded-md-sm', statusColor(slo.status))}>{slo.status}</span>
                </div>

                <div className="flex items-center gap-4 mb-3">
                  {/* Error Budget Ring */}
                  <div className="relative w-16 h-16 shrink-0">
                    <svg className="w-16 h-16 -rotate-90" viewBox="0 0 64 64">
                      <circle cx="32" cy="32" r="28" fill="none" stroke="var(--md-sys-color-surface-container-highest)" strokeWidth="4" />
                      <circle cx="32" cy="32" r="28" fill="none"
                        className={slo.status === 'breached' ? 'stroke-red-500' : slo.status === 'at_risk' ? 'stroke-amber-500' : 'stroke-green-500'}
                        strokeWidth="4" strokeDasharray={2 * Math.PI * 28}
                        strokeDashoffset={2 * Math.PI * 28 * (1 - budgetPercent / 100)}
                        strokeLinecap="round" />
                    </svg>
                    <div className="absolute inset-0 flex items-center justify-center">
                      <span className="text-xs font-bold text-md-on-surface">{budgetPercent.toFixed(0)}%</span>
                    </div>
                  </div>

                  <div className="flex-1 space-y-1">
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-label-small text-md-on-surface-variant">{t('slos.sli_type')}</span>
                      <span className="text-body-small font-medium text-md-on-surface">{slo.sli_type}</span>
                    </div>
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-label-small text-md-on-surface-variant">{t('slos.target')}</span>
                      <span className="text-body-small font-medium text-md-on-surface">{slo.target_percentage}%</span>
                    </div>
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-label-small text-md-on-surface-variant">{t('slos.current')}</span>
                      <span className="text-body-small font-medium text-md-on-surface">{slo.current_sli.toFixed(2)}%</span>
                    </div>
                  </div>
                </div>

                <div className="flex justify-end">
                  <button onClick={() => handleEvaluate(slo.id)}
                    className="text-xs px-3 py-1.5 rounded-md-full bg-md-primary/10 text-md-primary hover:bg-md-primary/20 transition-colors">
                    {t('slos.evaluate')}
                  </button>
                </div>
              </div>
            );
          })}
          {!loading && slos.length === 0 && (
            <div className="col-span-full glass-card rounded-md-xl p-8 text-center">
              <div className="text-4xl mb-3">📊</div>
              <p className="text-body-medium text-md-on-surface-variant">{t('slos.empty')}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
