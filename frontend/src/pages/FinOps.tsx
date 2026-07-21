import { useCallback, useEffect, useState } from 'react';
import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer } from 'recharts';
import { api } from '../api/client';
import type { CostOverview, CostByService, CostByProvider, CostBudget, CreateBudgetInput } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

const EMPTY_BUDGET: CreateBudgetInput = {
  name: '',
  amount: 0,
  period: 'monthly',
  start_date: '',
  end_date: '',
  notify_threshold: 80,
};

const COLORS = ['#6750A4', '#7D5260', '#B3261E', '#386A20', '#0061A4'];

export function FinOpsPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [overview, setOverview] = useState<CostOverview | null>(null);
  const [byService, setByService] = useState<CostByService[]>([]);
  const [byProvider, setByProvider] = useState<CostByProvider[]>([]);
  const [budgets, setBudgets] = useState<CostBudget[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showBudgetForm, setShowBudgetForm] = useState(false);
  const [budgetForm, setBudgetForm] = useState<CreateBudgetInput>(EMPTY_BUDGET);
  const [submitting, setSubmitting] = useState(false);

  const loadAll = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const [ov, svc, prov, bud] = await Promise.all([
        api.getFinOpsOverview(token),
        api.getFinOpsCostsByService(token),
        api.getFinOpsCostsByProvider(token),
        api.listFinOpsBudgets(token),
      ]);
      setOverview(ov);
      setByService(svc);
      setByProvider(prov);
      setBudgets(bud);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load data');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { loadAll(); }, [loadAll]);

  const handleCreateBudget = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    try {
      await api.createFinOpsBudget(token!, budgetForm);
      setBudgetForm(EMPTY_BUDGET);
      setShowBudgetForm(false);
      await loadAll();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create budget');
    } finally {
      setSubmitting(false);
    }
  };

  const handleDeleteBudget = async (id: string) => {
    if (!window.confirm(t('finops.delete_budget_confirm'))) return;
    try {
      await api.deleteFinOpsBudget(token!, id);
      await loadAll();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete budget');
    }
  };

  const formatCurrency = (amount: number) => `¥${amount.toLocaleString()}`;

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.finops')}
        </h2>
        <button onClick={loadAll} disabled={loading}
          className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors">
          {loading ? t('finops.loading') : t('finops.reload')}
        </button>
      </div>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}

      {/* Overview Cards */}
      {overview && (
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
          <div className="glass-card rounded-md-xl p-4">
            <p className="text-label-small text-md-on-surface-variant">{t('finops.this_month')}</p>
            <p className="text-headline-medium font-bold text-md-primary">{formatCurrency(overview.total_spend_this_month)}</p>
            <p className={cn('text-body-small', overview.month_over_month_change > 0 ? 'text-red-500' : 'text-green-500')}>
              {overview.month_over_month_change > 0 ? '↑' : '↓'} {Math.abs(overview.month_over_month_change).toFixed(1)}%
            </p>
          </div>
          <div className="glass-card rounded-md-xl p-4">
            <p className="text-label-small text-md-on-surface-variant">{t('finops.last_month')}</p>
            <p className="text-headline-medium font-bold text-md-on-surface">{formatCurrency(overview.total_spend_last_month)}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4">
            <p className="text-label-small text-md-on-surface-variant">{t('finops.forecast')}</p>
            <p className={cn('text-headline-medium font-bold', overview.forecast_spend > overview.budget_total ? 'text-red-500' : 'text-md-primary')}>
              {formatCurrency(overview.forecast_spend)}
            </p>
            {overview.budget_total > 0 && (
              <p className="text-body-small text-md-on-surface-variant">
                {t('finops.budget')}: {formatCurrency(overview.budget_total)}
              </p>
            )}
          </div>
          <div className="glass-card rounded-md-xl p-4">
            <p className="text-label-small text-md-on-surface-variant">{t('finops.budget_used')}</p>
            {overview.budget_total > 0 ? (
              <div>
                <p className="text-headline-medium font-bold text-md-on-surface">{((overview.budget_actual / overview.budget_total) * 100).toFixed(0)}%</p>
                <div className="mt-2 h-2 bg-md-surface-container-highest rounded-full overflow-hidden">
                  <div className={cn('h-full rounded-full transition-all', overview.budget_actual / overview.budget_total > 0.9 ? 'bg-red-500' : overview.budget_actual / overview.budget_total > 0.7 ? 'bg-amber-500' : 'bg-green-500')}
                    style={{ width: `${Math.min((overview.budget_actual / overview.budget_total) * 100, 100)}%` }} />
                </div>
              </div>
            ) : (
              <p className="text-body-small text-md-on-surface-variant">{t('finops.no_budget')}</p>
            )}
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* By Service Chart */}
        <div className="glass-card rounded-md-xl p-4">
          <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('finops.by_service')}</h3>
          {byService.length > 0 ? (
            <ResponsiveContainer width="100%" height={200}>
              <PieChart>
                <Pie data={byService.map(s => ({ name: s.service, value: s.total }))} cx="50%" cy="50%" outerRadius={80} dataKey="value" label>
                  {byService.map((_, index) => (
                    <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                  ))}
                </Pie>
                <Tooltip />
              </PieChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-[200px] flex items-center justify-center text-body-small text-md-on-surface-variant">{t('finops.no_data')}</div>
          )}
        </div>

        {/* By Provider Chart */}
        <div className="glass-card rounded-md-xl p-4">
          <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('finops.by_provider')}</h3>
          {byProvider.length > 0 ? (
            <ResponsiveContainer width="100%" height={200}>
              <PieChart>
                <Pie data={byProvider.map(p => ({ name: p.provider, value: p.total }))} cx="50%" cy="50%" outerRadius={80} dataKey="value" label>
                  {byProvider.map((_, index) => (
                    <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                  ))}
                </Pie>
                <Tooltip />
              </PieChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-[200px] flex items-center justify-center text-body-small text-md-on-surface-variant">{t('finops.no_data')}</div>
          )}
        </div>
      </div>

      {/* Budget Management */}
      <div className="glass-card rounded-md-xl p-4">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-title-medium font-semibold text-md-on-surface">{t('finops.budgets')}</h3>
          <button onClick={() => setShowBudgetForm(!showBudgetForm)}
            className="text-sm px-3 py-1.5 rounded-md-full bg-md-primary/10 text-md-primary hover:bg-md-primary/20 transition-colors">
            {showBudgetForm ? t('finops.cancel') : t('finops.add_budget')}
          </button>
        </div>

        {showBudgetForm && (
          <form onSubmit={handleCreateBudget} className="mb-4 p-3 bg-md-surface-container-highest rounded-md-lg space-y-2">
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
              <input type="text" required value={budgetForm.name} onChange={e => setBudgetForm({ ...budgetForm, name: e.target.value })}
                placeholder={t('finops.budget_name')} className="bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none" />
              <input type="number" required value={budgetForm.amount || ''} onChange={e => setBudgetForm({ ...budgetForm, amount: Number(e.target.value) })}
                placeholder={t('finops.budget_amount')} className="bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none" />
              <input type="date" required value={budgetForm.start_date} onChange={e => setBudgetForm({ ...budgetForm, start_date: e.target.value })}
                className="bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none" />
              <input type="date" required value={budgetForm.end_date} onChange={e => setBudgetForm({ ...budgetForm, end_date: e.target.value })}
                className="bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none" />
            </div>
            <div className="flex justify-end">
              <button type="submit" disabled={submitting} className="px-4 py-1.5 text-sm rounded-md-lg bg-md-primary text-md-on-primary hover:shadow-md-2 disabled:opacity-50 transition-all">
                {submitting ? t('finops.creating') : t('finops.create')}
              </button>
            </div>
          </form>
        )}

        <div className="space-y-2">
          {budgets.map(budget => (
            <div key={budget.id} className="flex items-center justify-between px-3 py-2 rounded-md-lg bg-md-surface-container-highest/50">
              <div className="flex items-center gap-3">
                <span className={cn('h-2 w-2 rounded-full', budget.status === 'on_track' ? 'bg-green-500' : budget.status === 'at_risk' ? 'bg-amber-500' : 'bg-red-500')} />
                <span className="text-body-medium font-medium text-md-on-surface">{budget.name}</span>
              </div>
              <div className="flex items-center gap-3">
                <span className="text-body-small text-md-on-surface-variant">{formatCurrency(budget.actual_spend)} / {formatCurrency(budget.amount)}</span>
                {budgets.length > 0 && (
                  <button onClick={() => handleDeleteBudget(budget.id)} className="text-xs px-2 py-1 rounded-md-sm text-md-error hover:bg-md-error-container/30 transition-colors">×</button>
                )}
              </div>
            </div>
          ))}
          {budgets.length === 0 && <p className="text-body-medium text-md-on-surface-variant text-center py-4">{t('finops.no_budgets')}</p>}
        </div>
      </div>
    </div>
  );
}
