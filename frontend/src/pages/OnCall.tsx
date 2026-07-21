import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { OnCallSchedule, OnCallShift, OnCallEscalation } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

export function OnCallPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [schedules, setSchedules] = useState<OnCallSchedule[]>([]);
  const [shifts, setShifts] = useState<OnCallShift[]>([]);
  const [escalations, setEscalations] = useState<OnCallEscalation[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState({ name: '', description: '', timezone: 'UTC', rotation_type: 'weekly' });
  const [submitting, setSubmitting] = useState(false);
  const [activeTab, setActiveTab] = useState<'schedules' | 'calendar' | 'escalations'>('schedules');

  const loadAll = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const [sch, sh, esc] = await Promise.all([
        api.listOnCallSchedules(token),
        api.listOnCallShifts(token),
        api.listOnCallEscalations(token),
      ]);
      setSchedules(sch);
      setShifts(sh);
      setEscalations(esc);
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
      await api.createOnCallSchedule(token!, form);
      setForm({ name: '', description: '', timezone: 'UTC', rotation_type: 'weekly' });
      setShowForm(false);
      await loadAll();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create schedule');
    } finally {
      setSubmitting(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!window.confirm(t('oncall.delete_confirm'))) return;
    try {
      await api.deleteOnCallSchedule(token!, id);
      await loadAll();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete');
    }
  };

  // Generate calendar grid for current month
  const now = new Date();
  const daysInMonth = new Date(now.getFullYear(), now.getMonth() + 1, 0).getDate();
  const firstDay = new Date(now.getFullYear(), now.getMonth(), 1).getDay();
  const calendarDays: (number | null)[] = [
    ...Array.from({ length: firstDay }, () => null),
    ...Array.from({ length: daysInMonth }, (_, i) => i + 1),
  ];

  // Map shifts to dates for calendar
  const shiftsByDay: Record<number, OnCallShift[]> = {};
  shifts.forEach(shift => {
    const startDate = new Date(shift.start_time);
    const endDate = new Date(shift.end_time);
    for (let d = new Date(startDate); d <= endDate; d.setDate(d.getDate() + 1)) {
      if (d.getMonth() === now.getMonth() && d.getFullYear() === now.getFullYear()) {
        const day = d.getDate();
        if (!shiftsByDay[day]) shiftsByDay[day] = [];
        shiftsByDay[day].push(shift);
      }
    }
  });


  if (loading) return <LoadingState skeleton="detail" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.oncall')}
        </h2>
        <div className="flex gap-2">
          <button onClick={loadAll} disabled={loading}
            className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors">
            {loading ? t('oncall.loading') : t('oncall.reload')}
          </button>
          {activeTab === 'schedules' && (
            <button onClick={() => setShowForm(!showForm)}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all">
              {showForm ? t('oncall.cancel') : t('oncall.add')}
            </button>
          )}
        </div>
      </div>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}

      {/* Tabs */}
      <div className="flex gap-2 border-b border-md-outline-variant pb-2">
        {(['schedules', 'calendar', 'escalations'] as const).map(tab => (
          <button key={tab} onClick={() => setActiveTab(tab)}
            className={cn('px-4 py-2 text-sm font-medium rounded-md-lg transition-colors',
              activeTab === tab ? 'bg-md-primary text-md-on-primary' : 'text-md-on-surface-variant hover:bg-md-surface-container-high')}>
            {t(`oncall.tab.${tab}`)}
          </button>
        ))}
      </div>

      {/* Create Form */}
      {showForm && activeTab === 'schedules' && (
        <form onSubmit={handleCreate} className="bg-md-surface-container-low rounded-md-lg p-4 space-y-3 animate-slide-up">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('oncall.name')}</label>
              <input type="text" required value={form.name} onChange={e => setForm({ ...form, name: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('oncall.rotation_type')}</label>
              <select value={form.rotation_type} onChange={e => setForm({ ...form, rotation_type: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface">
                <option value="weekly">{t('oncall.rotation_weekly')}</option>
                <option value="daily">{t('oncall.rotation_daily')}</option>
              </select>
            </div>
          </div>
          <div className="flex justify-end gap-2">
            <button type="button" onClick={() => setShowForm(false)} className="px-4 py-2 text-sm rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">{t('oncall.cancel')}</button>
            <button type="submit" disabled={submitting} className="px-4 py-2 text-sm rounded-md-lg bg-md-primary text-md-on-primary hover:shadow-md-2 disabled:opacity-50 transition-all">{submitting ? t('oncall.creating') : t('oncall.create')}</button>
          </div>
        </form>
      )}

      {/* Schedules */}
      {activeTab === 'schedules' && (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {schedules.map(s => (
            <div key={s.id} className="glass-card rounded-md-xl p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-body-medium font-medium text-md-on-surface">{s.name}</span>
                <span className={cn('h-2 w-2 rounded-full', s.enabled ? 'bg-green-500' : 'bg-md-outline')} />
              </div>
              <p className="text-body-small text-md-on-surface-variant mb-2">{s.description || t('oncall.no_description')}</p>
              <div className="flex items-center justify-between text-label-small text-md-on-surface-variant">
                <span>🔄 {s.rotation_type}</span>
                <span>🌍 {s.timezone}</span>
              </div>
              <div className="mt-3 flex justify-end">
                <button onClick={() => handleDelete(s.id)} className="text-xs px-2 py-1 rounded-md-sm text-md-error hover:bg-md-error-container/30 transition-colors">{t('oncall.delete')}</button>
              </div>
            </div>
          ))}
          {schedules.length === 0 && <div className="col-span-full glass-card rounded-md-xl p-8 text-center"><p className="text-body-medium text-md-on-surface-variant">{t('oncall.no_schedules')}</p></div>}
        </div>
      )}

      {/* Calendar */}
      {activeTab === 'calendar' && (
        <div className="glass-card rounded-md-xl p-4">
          <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">
            {now.toLocaleString('default', { month: 'long', year: 'numeric' })}
          </h3>
          <div className="grid grid-cols-7 gap-1">
            {['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'].map(d => (
              <div key={d} className="text-center text-label-small text-md-on-surface-variant py-1">{d}</div>
            ))}
            {calendarDays.map((day, i) => (
              <div key={i} className={cn(
                'aspect-square rounded-md-sm flex flex-col items-center justify-center text-body-small',
                day ? 'bg-md-surface-container-highest hover:bg-md-surface-container-high' : '',
                day === now.getDate() ? 'ring-2 ring-md-primary' : '',
              )}>
                {day && <span className="text-md-on-surface">{day}</span>}
                {day && shiftsByDay[day] && (
                  <div className="flex gap-0.5 mt-0.5">
                    {shiftsByDay[day].slice(0, 2).map((sh, j) => (
                      <div key={j} className={cn('w-1.5 h-1.5 rounded-full', sh.role === 'primary' ? 'bg-blue-500' : 'bg-amber-500')} />
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
          <div className="flex items-center gap-4 mt-3 text-label-small text-md-on-surface-variant">
            <span className="flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-blue-500" /> {t('oncall.primary')}</span>
            <span className="flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-amber-500" /> {t('oncall.secondary')}</span>
          </div>
        </div>
      )}

      {/* Escalations */}
      {activeTab === 'escalations' && (
        <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="border-b border-md-outline-variant">
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('oncall.esc_col.alert')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('oncall.esc_col.notified')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('oncall.esc_col.status')}</th>
                </tr>
              </thead>
              <tbody>
                {escalations.map(e => (
                  <tr key={e.id} className="border-b border-md-outline-variant last:border-0">
                    <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface">{e.alert_id}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{new Date(e.notified_at).toLocaleString()}</td>
                    <td className="whitespace-nowrap px-4 py-3">
                      <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm',
                        e.status === 'acknowledged' ? 'bg-green-500/10 text-green-600' :
                        e.status === 'missed' ? 'bg-red-500/10 text-red-600' :
                        'bg-amber-500/10 text-amber-600')}>{e.status}</span>
                    </td>
                  </tr>
                ))}
                {escalations.length === 0 && <tr><td colSpan={3} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('oncall.no_escalations')}</td></tr>}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
