import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { AlertRule, CreateAlertRuleInput } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

const EMPTY_FORM: CreateAlertRuleInput = {
  name: '',
  metric: 'cpu_percent',
  condition: '>',
  threshold: 90,
  severity: 'warning',
  silence_minutes: 5,
  channel_ids: [],
};

export function AlertRulesPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [rules, setRules] = useState<AlertRule[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<CreateAlertRuleInput>(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [editing, setEditing] = useState<string | null>(null);
  const [deleting, setDeleting] = useState<string | null>(null);

  const load = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    setError(null);
    try {
      const data = await api.listAlertRules(token);
      setRules(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load alert rules');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { load(); }, [load]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      if (editing) {
        await api.updateAlertRule(token!, editing, form);
      } else {
        await api.createAlertRule(token!, form);
      }
      setForm(EMPTY_FORM);
      setShowForm(false);
      setEditing(null);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to save rule');
    } finally {
      setSubmitting(false);
    }
  };

  const handleEdit = (rule: AlertRule) => {
    setForm({
      name: rule.name,
      metric: rule.metric,
      condition: rule.condition,
      threshold: rule.threshold,
      severity: rule.severity,
      silence_minutes: rule.silence_minutes,
    });
    setEditing(rule.id);
    setShowForm(true);
  };

  const handleToggle = async (rule: AlertRule) => {
    if (!token) return;
    try {
      await api.updateAlertRule(token, rule.id, { enabled: !rule.enabled });
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to toggle rule');
    }
  };

  const handleDelete = async (id: string) => {
    if (!window.confirm(t('alert_rules.delete_confirm'))) return;
    setDeleting(id);
    try {
      await api.deleteAlertRule(token!, id);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete rule');
    } finally {
      setDeleting(null);
    }
  };

  const severityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      case 'warning': return 'bg-amber-500/10 text-amber-600 dark:text-amber-400';
      case 'info': return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.alert_rules')}
        </h2>
        <div className="flex gap-2">
          <button
            onClick={load}
            disabled={loading}
            className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors"
          >
            {loading ? t('alert_rules.loading') : t('alert_rules.reload')}
          </button>
          <button
            onClick={() => { setShowForm(!showForm); setEditing(null); setForm(EMPTY_FORM); }}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all"
          >
            {showForm ? t('alert_rules.cancel') : t('alert_rules.add')}
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {showForm && (
        <form onSubmit={handleSubmit} className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 space-y-3 animate-slide-up">
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('alert_rules.name')}</label>
              <input type="text" required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface"
                placeholder="High CPU Alert" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('alert_rules.metric')}</label>
              <select value={form.metric} onChange={(e) => setForm({ ...form, metric: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface">
                <option value="cpu_percent">CPU %</option>
                <option value="memory_percent">Memory %</option>
                <option value="disk_percent">Disk %</option>
                <option value="network_in">Network In</option>
                <option value="network_out">Network Out</option>
              </select>
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('alert_rules.condition')}</label>
              <div className="flex gap-2">
                <select value={form.condition} onChange={(e) => setForm({ ...form, condition: e.target.value })}
                  className="w-20 bg-md-surface-container-highest rounded-md-sm px-3 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface">
                  <option value=">">&gt;</option>
                  <option value=">=">&gt;=</option>
                  <option value="<">&lt;</option>
                  <option value="<=">&lt;=</option>
                  <option value="==">==</option>
                </select>
                <input type="number" required value={form.threshold} onChange={(e) => setForm({ ...form, threshold: Number(e.target.value) })}
                  className="flex-1 bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface"
                  placeholder="90" />
              </div>
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('alert_rules.severity')}</label>
              <select value={form.severity} onChange={(e) => setForm({ ...form, severity: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface">
                <option value="critical">{t('alert_rules.severity_critical')}</option>
                <option value="warning">{t('alert_rules.severity_warning')}</option>
                <option value="info">{t('alert_rules.severity_info')}</option>
              </select>
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('alert_rules.silence_minutes')}</label>
              <input type="number" value={form.silence_minutes} onChange={(e) => setForm({ ...form, silence_minutes: Number(e.target.value) })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface"
                placeholder="5" />
            </div>
          </div>
          <div className="flex justify-end gap-2">
            <button type="button" onClick={() => { setShowForm(false); setEditing(null); }}
              className="px-4 py-2 text-sm font-medium rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">
              {t('alert_rules.cancel')}
            </button>
            <button type="submit" disabled={submitting}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
              {submitting ? t('alert_rules.saving') : t('alert_rules.save')}
            </button>
          </div>
        </form>
      )}

      <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
        <div className="overflow-x-auto">
          <table className="min-w-full">
            <thead>
              <tr className="border-b border-md-outline-variant">
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('alert_rules.col.name')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('alert_rules.col.condition')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('alert_rules.col.severity')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('alert_rules.col.silence')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('alert_rules.col.status')}</th>
                <th className="px-4 py-3 text-right text-label-medium text-md-on-surface-variant">{t('alert_rules.col.actions')}</th>
              </tr>
            </thead>
            <tbody>
              {rules.map((rule) => (
                <tr key={rule.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{rule.name}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant font-mono">
                    {rule.metric} {rule.condition} {rule.threshold}
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small">
                    <span className={cn('inline-block text-xs font-medium px-2 py-1 rounded-md-sm', severityColor(rule.severity))}>
                      {rule.severity}
                    </span>
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{rule.silence_minutes}m</td>
                  <td className="whitespace-nowrap px-4 py-3">
                    <button
                      onClick={() => handleToggle(rule)}
                      className={cn(
                        'relative w-10 h-6 rounded-full transition-colors',
                        rule.enabled ? 'bg-md-primary' : 'bg-md-surface-container-highest',
                      )}
                    >
                      <div className={cn(
                        'absolute top-1 w-4 h-4 rounded-full bg-white shadow transition-transform',
                        rule.enabled ? 'translate-x-5' : 'translate-x-1',
                      )} />
                    </button>
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-right">
                    <div className="flex items-center justify-end gap-2">
                      <button onClick={() => handleEdit(rule)}
                        className="text-md-primary text-label-large hover:bg-md-primary-container/30 px-2 py-1 rounded-md-sm transition-colors">
                        {t('alert_rules.edit')}
                      </button>
                      <button onClick={() => handleDelete(rule.id)} disabled={deleting === rule.id}
                        className="text-md-error text-label-large hover:bg-md-error-container/30 px-2 py-1 rounded-md-sm disabled:opacity-50 transition-colors">
                        {deleting === rule.id ? t('alert_rules.deleting') : t('alert_rules.delete')}
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
              {!loading && rules.length === 0 && (
                <tr><td colSpan={6} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('alert_rules.empty')}</td></tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
