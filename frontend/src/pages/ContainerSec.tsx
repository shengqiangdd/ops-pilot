import { useState } from 'react';
import { api } from '../api/client';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

export function ContainerSecPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();
  const [imageName, setImageName] = useState('');
  const [scanResult, setScanResult] = useState<any>(null);
  const [runtimeResult, setRuntimeResult] = useState<any>(null);
  const [loading, setLoading] = useState(false);

  const handleScan = async () => {
    if (!token || !imageName) return;
    setLoading(true);
    try {
      const data = await api.scanContainerImage(token, imageName);
      setScanResult(data);
    } catch (e) { console.error(e); }
    setLoading(false);
  };

  const handleRuntimeCheck = async () => {
    if (!token) return;
    setLoading(true);
    try {
      const data = await api.checkContainerRuntime(token);
      setRuntimeResult(data);
    } catch (e) { console.error(e); }
    setLoading(false);
  };

  const severityColor = (sev: string) => {
    switch (sev) {
      case 'critical': return 'bg-red-500/10 text-red-600';
      case 'high': return 'bg-orange-500/10 text-orange-600';
      case 'medium': return 'bg-amber-500/10 text-amber-600';
      default: return 'bg-green-500/10 text-green-600';
    }
  };

  return (
    <div className="space-y-4 animate-slide-up">
      <h2 className="text-headline-small font-medium text-md-on-surface">{t('title.container_sec')}</h2>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Image Scanner */}
        <div className="glass-card rounded-md-xl p-5">
          <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('container_sec.image_scan')}</h3>
          <div className="flex gap-3 mb-4">
            <input value={imageName} onChange={e => setImageName(e.target.value)} className="flex-1 bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" placeholder="nginx:latest" />
            <button onClick={handleScan} disabled={loading} className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 text-sm font-medium hover:shadow-md-2 transition-all disabled:opacity-50">
              {loading ? t('container_sec.scanning') : t('container_sec.scan_btn')}
            </button>
          </div>
          {scanResult && (
            <div className="space-y-2">
              <div className="flex items-center gap-2 mb-3">
                <span className="text-headline-medium font-bold text-md-primary">{scanResult.score.toFixed(0)}</span>
                <span className="text-body-small text-md-on-surface-variant">/ 100</span>
              </div>
              {scanResult.issues?.map((issue: any, i: number) => (
                <div key={i} className={cn('px-3 py-2 rounded-md-lg text-sm', severityColor(issue.severity))}>
                  <p className="font-medium">{issue.check}</p>
                  <p className="text-md-on-surface-variant">{issue.description}</p>
                  <p className="text-label-small text-md-on-surface-variant mt-1">→ {issue.recommendation}</p>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Runtime Checker */}
        <div className="glass-card rounded-md-xl p-5">
          <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('container_sec.runtime_check')}</h3>
          <button onClick={handleRuntimeCheck} disabled={loading} className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 text-sm font-medium hover:shadow-md-2 transition-all disabled:opacity-50 mb-4">
            {loading ? t('container_sec.checking') : t('container_sec.check_btn')}
          </button>
          {runtimeResult && (
            <div className="space-y-3">
              <div className="flex items-center gap-2 mb-3">
                <span className="text-headline-medium font-bold text-md-primary">{runtimeResult.score.toFixed(0)}</span>
                <span className="text-body-small text-md-on-surface-variant">/ 100</span>
              </div>
              {runtimeResult.checks?.map((check: any, i: number) => (
                <div key={i} className="flex items-center gap-3 px-3 py-2 rounded-md-lg bg-md-surface-container-highest/50">
                  <span className={cn('h-5 w-5 rounded-full flex items-center justify-center text-xs', check.passed ? 'bg-green-500/20 text-green-600' : 'bg-red-500/20 text-red-600')}>
                    {check.passed ? '✓' : '✗'}
                  </span>
                  <span className="flex-1 text-body-small font-medium text-md-on-surface">{check.name}</span>
                  <span className="text-label-small text-md-on-surface-variant">{check.description}</span>
                </div>
              ))}
              {runtimeResult.recommendations?.length > 0 && (
                <div className="mt-3 space-y-1">
                  <p className="text-label-medium text-md-on-surface-variant">{t('container_sec.recommendations')}</p>
                  {runtimeResult.recommendations.map((rec: string, i: number) => (
                    <p key={i} className="text-body-small text-md-on-surface">• {rec}</p>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
