import { useState } from 'react';
import { api } from '../api/client';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

export function InspectionPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();
  const [taskName, setTaskName] = useState('');
  const [selectedCategories, setSelectedCategories] = useState<string[]>(['health', 'security']);
  const [result, setResult] = useState<any>(null);
  const [loading, setLoading] = useState(false);

  const categories = ['health', 'process', 'certificate', 'security', 'log', 'network'];

  const handleCreateAndRun = async () => {
    if (!token || !taskName) return;
    setLoading(true);
    try {
      const task = await api.createInspection(token, { name: taskName, categories: selectedCategories });
      const res = await api.runInspection(token, task.id);
      setResult(res);
    } catch (e) { console.error(e); }
    setLoading(false);
  };

  const toggleCategory = (cat: string) => {
    setSelectedCategories(prev => prev.includes(cat) ? prev.filter(c => c !== cat) : [...prev, cat]);
  };

  return (
    <div className="space-y-4 animate-slide-up">
      <h2 className="text-headline-small font-medium text-md-on-surface">{t('title.inspection')}</h2>

      <div className="glass-card rounded-md-xl p-5">
        <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('inspection.create')}</h3>
        <div className="mb-3">
          <label className="block text-label-large text-md-on-surface mb-1">{t('inspection.name')}</label>
          <input value={taskName} onChange={e => setTaskName(e.target.value)} className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" placeholder={t('inspection.name_placeholder')} />
        </div>
        <div className="mb-4">
          <label className="block text-label-large text-md-on-surface mb-2">{t('inspection.categories')}</label>
          <div className="flex flex-wrap gap-2">
            {categories.map(cat => (
              <button key={cat} onClick={() => toggleCategory(cat)} className={cn('px-3 py-1.5 text-sm rounded-md-full transition-colors', selectedCategories.includes(cat) ? 'bg-md-primary text-md-on-primary' : 'bg-md-surface-container-high text-md-on-surface-variant')}>
                {cat}
              </button>
            ))}
          </div>
        </div>
        <button onClick={handleCreateAndRun} disabled={loading} className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 text-sm font-medium hover:shadow-md-2 transition-all disabled:opacity-50">
          {loading ? t('inspection.running') : t('inspection.run_btn')}
        </button>
      </div>

      {result && (
        <div className="glass-card rounded-md-xl p-5">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-title-medium font-semibold text-md-on-surface">{t('inspection.result')}</h3>
            <span className={cn('px-3 py-1 rounded-full text-sm font-medium', result.status === 'pass' ? 'bg-green-500/10 text-green-600' : 'bg-red-500/10 text-red-600')}>
              {result.score.toFixed(0)}%
            </span>
          </div>
          <p className="text-body-medium text-md-on-surface-variant mb-3">{result.summary}</p>
          <div className="space-y-2">
            {result.item_results?.map((item: any, i: number) => (
              <div key={i} className="flex items-center gap-3 px-3 py-2 rounded-md-lg bg-md-surface-container-highest/50">
                <span className={cn('h-5 w-5 rounded-full flex items-center justify-center text-xs', item.passed ? 'bg-green-500/20 text-green-600' : 'bg-red-500/20 text-red-600')}>
                  {item.passed ? '✓' : '✗'}
                </span>
                <div className="flex-1">
                  <span className="text-body-small font-medium text-md-on-surface">{item.check_name}</span>
                  <p className="text-label-small text-md-on-surface-variant">{item.message}</p>
                </div>
                <span className="text-label-small text-md-on-surface-variant">{item.actual_value}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
