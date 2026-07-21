import { useState } from 'react';
import { api } from '../api/client';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

export function ChangeRiskPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();
  const [resource, setResource] = useState('');
  const [changeType, setChangeType] = useState('config_change');
  const [description, setDescription] = useState('');
  const [affectedServices, setAffectedServices] = useState('');
  const [result, setResult] = useState<any>(null);
  const [loading, setLoading] = useState(false);

  const handleAssess = async () => {
    if (!token) return;
    setLoading(true);
    try {
      const data = await api.analyzeChangeRisk(token, {
        resource, change_type: changeType, description,
        affected_services: affectedServices.split(',').map(s => s.trim()).filter(Boolean),
      });
      setResult(data);
    } catch (e) { console.error(e); }
    setLoading(false);
  };

  const levelColor = (level: string) => {
    switch (level) {
      case 'critical': return 'bg-red-500/10 text-red-600';
      case 'high': return 'bg-orange-500/10 text-orange-600';
      case 'medium': return 'bg-amber-500/10 text-amber-600';
      default: return 'bg-green-500/10 text-green-600';
    }
  };

  return (
    <div className="space-y-4 animate-slide-up">
      <h2 className="text-headline-small font-medium text-md-on-surface">{t('title.change_risk')}</h2>

      <div className="glass-card rounded-md-xl p-5">
        <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('change_risk.assess')}</h3>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 mb-4">
          <div>
            <label className="block text-label-large text-md-on-surface mb-1">{t('change_risk.resource')}</label>
            <input value={resource} onChange={e => setResource(e.target.value)} className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" placeholder="host/prod-1" />
          </div>
          <div>
            <label className="block text-label-large text-md-on-surface mb-1">{t('change_risk.type')}</label>
            <select value={changeType} onChange={e => setChangeType(e.target.value)} className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface">
              <option value="config_change">{t('change_risk.type_config')}</option>
              <option value="deployment">{t('change_risk.type_deploy')}</option>
              <option value="restart">{t('change_risk.type_restart')}</option>
              <option value="read_only">{t('change_risk.type_readonly')}</option>
            </select>
          </div>
        </div>
        <div className="mb-4">
          <label className="block text-label-large text-md-on-surface mb-1">{t('change_risk.description')}</label>
          <textarea value={description} onChange={e => setDescription(e.target.value)} rows={2} className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
        </div>
        <div className="mb-4">
          <label className="block text-label-large text-md-on-surface mb-1">{t('change_risk.services')}</label>
          <input value={affectedServices} onChange={e => setAffectedServices(e.target.value)} className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" placeholder="svc-web,svc-api (comma separated)" />
        </div>
        <button onClick={handleAssess} disabled={loading} className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 text-sm font-medium hover:shadow-md-2 transition-all disabled:opacity-50">
          {loading ? t('change_risk.assessing') : t('change_risk.assess_btn')}
        </button>
      </div>

      {result && (
        <div className="glass-card rounded-md-xl p-5">
          <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('change_risk.result')}</h3>
          <div className="flex items-center gap-4 mb-4">
            <div className="text-headline-large font-bold text-md-primary">{(result.score * 100).toFixed(0)}</div>
            <span className={cn('px-3 py-1 rounded-full text-sm font-medium', levelColor(result.level))}>{result.level}</span>
          </div>
          <p className="text-body-medium text-md-on-surface-variant mb-3">{result.recommendation}</p>
          {result.factors?.length > 0 && (
            <div className="space-y-2">
              {result.factors.map((f: any, i: number) => (
                <div key={i} className="flex items-center gap-2 text-sm">
                  <span className="text-md-on-surface-variant">{f.category}:</span>
                  <span className="text-md-on-surface">{f.description}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
