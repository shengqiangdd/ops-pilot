import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { RemediationRule, RemediationExecution, CreateRemediationRuleInput } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

const EMPTY_FORM: CreateRemediationRuleInput = {
  name: '',
  trigger_type: 'alert',
  actions_json: '[{"type":"restart_service","host_id":"host-001","service_name":"nginx"}]',
  cooldown_minutes: 30,
  max_retries: 3,
};

export function RemediationPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [rules, setRules] = useState<RemediationRule[]>([]);
  const [executions, setExecutions] = useState<RemediationExecution[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<CreateRemediationRuleInput>(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [activeTab, setActiveTab] = useState<'rules' | 'executions'>('rules');

  const loadRules = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const data = await api.listRemediationRules(token);
      setRules(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load rules');
    } finally {
      setLoading(false);
    }
  }, [token]);

  const loadExecutions = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.listRemediationExecutions(token);
      setExecutions(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load executions');
    }
  }, [token]);

  useEffect(() => {
    if (activeTab === 'rules') loadRules();
    else loadExecutions();
  }, [activeTab, loadRules, loadExecutions]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    try {
      await api.createRemediationRule(token!, form);
      setForm(EMPTY_FORM);
      setShowForm(false);
      await loadRules();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create rule');
    } finally {
      setSubmitting(false);
    }
  };

  const handleTestRule = async (ruleId: string) => {
    try {
      await api.testRemediationRule(token!, ruleId);
      await loadExecutions();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to test rule');
    }
  };

  const handleDeleteRule = async (id: string) => {
    if (!window.confirm(t('remediation.delete_confirm'))) return;
    try {
      await api.deleteRemediationRule(token!, id);
      await loadRules();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete rule');
    }
  };

  const statusColor = (status: string) => {
    switch (status) {
      case 'success': case 'completed': return 'bg-green-500/10 text-green-600 dark:text-green-400';
      case 'running': case 'pending': return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
      case 'failed': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      case 'skipped': return 'bg-md-surface-container-high text-md-on-surface-variant';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.remediation')}
        </h2>
        <div className="flex gap-2">
          <button onClick={() => activeTab === 'rules' ? loadRules() : loadExecutions()}
            disabled={loading}
            className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors">
            {loading ? t('remediation.loading') : t('remediation.reload')}
          </button>
          {activeTab === 'rules' && (
            <button onClick={() => setShowForm(!showForm)}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all">
              {showForm ? t('remediation.cancel') : t('remediation.add')}
            </button>
          )}
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {/* Tabs */}
      <div className="flex gap-2 border-b border-md-outline-variant pb-2">
        <button onClick={() => setActiveTab('rules')}
          className={cn('px-4 py-2 text-sm font-medium rounded-md-lg transition-colors',
            activeTab === 'rules' ? 'bg-md-primary text-md-on-primary' : 'text-md-on-surface-variant hover:bg-md-surface-container-high')}>
          {t('remediation.tab.rules')}
        </button>
        <button onClick={() => setActiveTab('executions')}
          className={cn('px-4 py-2 text-sm font-medium rounded-md-lg transition-colors',
            activeTab === 'executions' ? 'bg-md-primary text-md-on-primary' : 'text-md-on-surface-variant hover:bg-md-surface-container-high')}>
          {t('remediation.tab.executions')}
        </button>
      </div>

      {/* Create Form */}
      {showForm && activeTab === 'rules' && (
        <form onSubmit={handleSubmit} className="bg-md-surface-container-low rounded-md-lg p-4 space-y-3 animate-slide-up">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('remediation.name')}</label>
              <input type="text" required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('remediation.trigger_type')}</label>
              <select value={form.trigger_type} onChange={(e) => setForm({ ...form, trigger_type: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface">
                <option value="alert">{t('remediation.trigger_alert')}</option>
                <option value="incident">{t('remediation.trigger_incident')}</option>
              </select>
            </div>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('remediation.cooldown')}</label>
              <input type="number" min="0" value={form.cooldown_minutes} onChange={(e) => setForm({ ...form, cooldown_minutes: Number(e.target.value) })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('remediation.max_retries')}</label>
              <input type="number" min="0" max="10" value={form.max_retries} onChange={(e) => setForm({ ...form, max_retries: Number(e.target.value) })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
          </div>
          <div>
            <label className="block text-label-large text-md-on-surface mb-1">{t('remediation.actions')}</label>
            <textarea value={form.actions_json} onChange={(e) => setForm({ ...form, actions_json: e.target.value })}
              rows={4}
              className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface font-mono text-sm" />
          </div>
          <div className="flex justify-end gap-2">
            <button type="button" onClick={() => setShowForm(false)}
              className="px-4 py-2 text-sm rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">
              {t('remediation.cancel')}
            </button>
            <button type="submit" disabled={submitting}
              className="px-4 py-2 text-sm rounded-md-lg bg-md-primary text-md-on-primary hover:shadow-md-2 disabled:opacity-50 transition-all">
              {submitting ? t('remediation.creating') : t('remediation.create')}
            </button>
          </div>
        </form>
      )}

      {/* Rules Table */}
      {activeTab === 'rules' && (
        <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="border-b border-md-outline-variant">
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('remediation.rule_col.name')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('remediation.rule_col.trigger')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('remediation.rule_col.cooldown')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('remediation.rule_col.retries')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('remediation.rule_col.status')}</th>
                  <th className="px-4 py-3 text-right text-label-medium text-md-on-surface-variant">{t('remediation.rule_col.actions')}</th>
                </tr>
              </thead>
              <tbody>
                {rules.map((rule) => (
                  <tr key={rule.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                    <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{rule.name}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{rule.trigger_type}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{rule.cooldown_minutes}m</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{rule.max_retries}</td>
                    <td className="whitespace-nowrap px-4 py-3">
                      <span className={cn('h-2 w-2 rounded-full inline-block', rule.enabled ? 'bg-green-500' : 'bg-md-outline')} />
                    </td>
                    <td className="whitespace-nowrap px-4 py-3 text-right">
                      <div className="flex items-center justify-end gap-2">
                        <button onClick={() => handleTestRule(rule.id)}
                          className="text-md-primary text-label-large hover:bg-md-primary-container/30 px-2 py-1 rounded-md-sm transition-colors">
                          {t('remediation.test')}
                        </button>
                        <button onClick={() => handleDeleteRule(rule.id)}
                          className="text-md-error text-label-large hover:bg-md-error-container/30 px-2 py-1 rounded-md-sm transition-colors">
                          {t('remediation.delete')}
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
                {rules.length === 0 && (
                  <tr><td colSpan={6} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('remediation.no_rules')}</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Executions Table */}
      {activeTab === 'executions' && (
        <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="border-b border-md-outline-variant">
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('remediation.exec_col.rule')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('remediation.exec_col.trigger')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('remediation.exec_col.status')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('remediation.exec_col.time')}</th>
                </tr>
              </thead>
              <tbody>
                {executions.map((exec) => (
                  <tr key={exec.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                    <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface font-mono">{exec.rule_id}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{exec.trigger_type}</td>
                    <td className="whitespace-nowrap px-4 py-3">
                      <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', statusColor(exec.status))}>{exec.status}</span>
                    </td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{exec.started_at || '-'}</td>
                  </tr>
                ))}
                {executions.length === 0 && (
                  <tr><td colSpan={4} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('remediation.no_executions')}</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
