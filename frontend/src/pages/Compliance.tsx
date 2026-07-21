import { useCallback, useEffect, useState } from 'react';
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip } from 'recharts';
import { api } from '../api/client';
import type { ComplianceFramework, ComplianceOverview } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

export function CompliancePage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [frameworks, setFrameworks] = useState<ComplianceFramework[]>([]);
  const [overview, setOverview] = useState<ComplianceOverview | null>(null);
  const [selectedFramework, setSelectedFramework] = useState('cis-benchmark');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);

  const loadFrameworks = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.listComplianceFrameworks(token);
      setFrameworks(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load frameworks');
    }
  }, [token]);

  const loadOverview = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.getComplianceOverview(token);
      setOverview(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load overview');
    }
  }, [token]);

  useEffect(() => {
    Promise.all([loadFrameworks(), loadOverview()]).finally(() => setLoading(false));
  }, [loadFrameworks, loadOverview]);

  const handleScan = async () => {
    setScanning(true);
    try {
      await api.runComplianceScan(token!, { framework_id: selectedFramework });
      await loadOverview();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to scan');
    } finally {
      setScanning(false);
    }
  };

  const radius = 60;
  const circumference = 2 * Math.PI * radius;
  const passRate = overview?.pass_rate ?? 0;
  const progress = (passRate / 100) * circumference;

  const pieData = overview ? [
    { name: 'Passed', value: overview.passed, color: '#4CAF50' },
    { name: 'Failed', value: overview.failed, color: '#B3261E' },
    { name: 'N/A', value: overview.not_applicable, color: '#9E9E9E' },
  ].filter(d => d.value > 0) : [];

  if (loading) return <LoadingState skeleton="list" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.compliance')}
        </h2>
        <button onClick={handleScan} disabled={scanning}
          className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50 flex items-center gap-2">
          {scanning ? (
            <div className="h-4 w-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
          ) : '🔍'}
          {scanning ? t('compliance.scanning') : t('compliance.scan')}
        </button>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {/* Framework Tabs */}
      <div className="flex gap-2 overflow-x-auto pb-2">
        {frameworks.map((fw) => (
          <button
            key={fw.id}
            onClick={() => setSelectedFramework(fw.id)}
            className={cn(
              'px-4 py-2 text-sm font-medium rounded-md-full whitespace-nowrap transition-colors',
              selectedFramework === fw.id
                ? 'bg-md-primary text-md-on-primary'
                : 'bg-md-surface-container-high text-md-on-surface-variant hover:bg-md-surface-container-highest',
            )}
          >
            {fw.name} {fw.version}
          </button>
        ))}
        {frameworks.length === 0 && (
          <span className="text-body-small text-md-on-surface-variant">{t('compliance.no_frameworks')}</span>
        )}
      </div>

      {/* Overview */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Pass Rate Circle */}
        <div className="glass-card rounded-md-xl p-6 flex flex-col items-center justify-center">
          <div className="relative w-32 h-32 mb-2">
            <svg className="w-32 h-32 -rotate-90" viewBox="0 0 140 140">
              <circle cx="70" cy="70" r={radius} fill="none" stroke="var(--md-sys-color-surface-container-highest)" strokeWidth="8" />
              <circle cx="70" cy="70" r={radius} fill="none"
                className={cn('transition-all duration-1000', passRate >= 80 ? 'stroke-green-500' : passRate >= 60 ? 'stroke-amber-500' : 'stroke-red-500')}
                strokeWidth="8" strokeDasharray={circumference} strokeDashoffset={circumference - progress} strokeLinecap="round" />
            </svg>
            <div className="absolute inset-0 flex flex-col items-center justify-center">
              <span className="text-headline-large font-bold text-md-on-surface">{passRate.toFixed(0)}%</span>
              <span className="text-label-small text-md-on-surface-variant">{t('compliance.pass_rate')}</span>
            </div>
          </div>
        </div>

        {/* Stats */}
        <div className="glass-card rounded-md-xl p-6">
          <h3 className="text-title-small font-semibold text-md-on-surface mb-3">{t('compliance.summary')}</h3>
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-body-small text-md-on-surface-variant">{t('compliance.total')}</span>
              <span className="text-body-medium font-medium text-md-on-surface">{overview?.total_controls ?? 0}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-body-small text-md-on-surface-variant">{t('compliance.passed')}</span>
              <span className="text-body-medium font-medium text-green-500">{overview?.passed ?? 0}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-body-small text-md-on-surface-variant">{t('compliance.failed')}</span>
              <span className="text-body-medium font-medium text-red-500">{overview?.failed ?? 0}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-body-small text-md-on-surface-variant">{t('compliance.not_applicable')}</span>
              <span className="text-body-medium font-medium text-md-on-surface-variant">{overview?.not_applicable ?? 0}</span>
            </div>
          </div>
        </div>

        {/* Pie Chart */}
        <div className="glass-card rounded-md-xl p-4">
          <h3 className="text-title-small font-semibold text-md-on-surface mb-2">{t('compliance.distribution')}</h3>
          {pieData.length > 0 ? (
            <ResponsiveContainer width="100%" height={150}>
              <PieChart>
                <Pie data={pieData} cx="50%" cy="50%" outerRadius={60} dataKey="value" label>
                  {pieData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip />
              </PieChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-[150px] flex items-center justify-center text-body-small text-md-on-surface-variant">
              {t('compliance.no_data')}
            </div>
          )}
        </div>
      </div>

      {/* Category Breakdown */}
      {overview && overview.by_category.length > 0 && (
        <div className="glass-card rounded-md-xl p-5">
          <h3 className="text-title-medium font-semibold text-md-on-surface mb-4">{t('compliance.by_category')}</h3>
          <div className="space-y-3">
            {overview.by_category.map((cat) => {
              const catPassRate = cat.total > 0 ? (cat.passed / cat.total) * 100 : 0;
              return (
                <div key={cat.category}>
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-body-small font-medium text-md-on-surface">{cat.category}</span>
                    <span className="text-label-small text-md-on-surface-variant">{cat.passed}/{cat.total}</span>
                  </div>
                  <div className="h-2 bg-md-surface-container-highest rounded-full overflow-hidden">
                    <div className={cn('h-full rounded-full transition-all', catPassRate >= 80 ? 'bg-green-500' : catPassRate >= 60 ? 'bg-amber-500' : 'bg-red-500')} style={{ width: `${catPassRate}%` }} />
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
