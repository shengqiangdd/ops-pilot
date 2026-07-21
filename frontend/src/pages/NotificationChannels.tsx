import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { NotificationChannel, CreateChannelInput } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

const EMPTY_FORM: CreateChannelInput = {
  name: '',
  channel_type: 'webhook',
  config: {},
};

export function NotificationChannelsPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [channels, setChannels] = useState<NotificationChannel[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<CreateChannelInput>(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [testing, setTesting] = useState<string | null>(null);

  const load = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    setError(null);
    try {
      const data = await api.listNotificationChannels(token);
      setChannels(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load channels');
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
      await api.createNotificationChannel(token!, form);
      setForm(EMPTY_FORM);
      setShowForm(false);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create channel');
    } finally {
      setSubmitting(false);
    }
  };

  const handleTest = async (channelId: string) => {
    setTesting(channelId);
    try {
      const resp = await api.testNotificationChannel(token!, channelId);
      alert(resp.status === 'ok' ? t('channels.test_success') : t('channels.test_failed'));
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Test failed');
    } finally {
      setTesting(null);
    }
  };

  const typeIcon = (type: string) => {
    switch (type) {
      case 'webhook': return '🔗';
      case 'email': return '📧';
      case 'dingtalk': return '💬';
      case 'wecom': return '💼';
      default: return '📢';
    }
  };


  if (loading) return <LoadingState skeleton="list" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.channels')}
        </h2>
        <div className="flex gap-2">
          <button
            onClick={load}
            disabled={loading}
            className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors"
          >
            {loading ? t('channels.loading') : t('channels.reload')}
          </button>
          <button
            onClick={() => setShowForm(!showForm)}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all"
          >
            {showForm ? t('channels.cancel') : t('channels.add')}
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {showForm && (
        <form onSubmit={handleSubmit} className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 space-y-3 animate-slide-up">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('channels.name')}</label>
              <input type="text" required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface"
                placeholder="My Webhook" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('channels.type')}</label>
              <select value={form.channel_type} onChange={(e) => setForm({ ...form, channel_type: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface">
                <option value="webhook">Webhook</option>
                <option value="email">Email</option>
                <option value="dingtalk">DingTalk</option>
                <option value="wecom">WeCom</option>
              </select>
            </div>
            <div className="sm:col-span-2">
              <label className="block text-label-large text-md-on-surface mb-1">{t('channels.config')}</label>
              <textarea value={JSON.stringify(form.config, null, 2)}
                onChange={(e) => {
                  try { setForm({ ...form, config: JSON.parse(e.target.value) }); }
                  catch { /* ignore */ }
                }}
                rows={4}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface font-mono text-sm"
                placeholder='{"url": "https://hooks.example.com/xxx"}' />
            </div>
          </div>
          <div className="flex justify-end gap-2">
            <button type="button" onClick={() => setShowForm(false)}
              className="px-4 py-2 text-sm font-medium rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">
              {t('channels.cancel')}
            </button>
            <button type="submit" disabled={submitting}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
              {submitting ? t('channels.creating') : t('channels.create_btn')}
            </button>
          </div>
        </form>
      )}

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {channels.map((ch) => (
          <div key={ch.id} className="glass-card rounded-md-xl p-4 flex flex-col">
            <div className="flex items-center gap-3 mb-3">
              <span className="text-2xl">{typeIcon(ch.channel_type)}</span>
              <div className="flex-1 min-w-0">
                <h3 className="text-title-small font-semibold text-md-on-surface truncate">{ch.name}</h3>
                <p className="text-label-small text-md-on-surface-variant">{ch.channel_type}</p>
              </div>
              <div className={cn(
                'relative w-10 h-6 rounded-full transition-colors',
                ch.enabled ? 'bg-md-primary' : 'bg-md-surface-container-highest',
              )}>
                <div className={cn(
                  'absolute top-1 w-4 h-4 rounded-full bg-white shadow transition-transform',
                  ch.enabled ? 'translate-x-5' : 'translate-x-1',
                )} />
              </div>
            </div>
            <div className="flex-1">
              <pre className="text-body-small text-md-on-surface-variant bg-md-surface-container-highest rounded-md-sm p-2 overflow-auto max-h-24 font-mono">
                {ch.config}
              </pre>
            </div>
            <div className="mt-3 flex justify-end">
              <button
                onClick={() => handleTest(ch.id)}
                disabled={testing === ch.id}
                className="text-md-primary text-label-large hover:bg-md-primary-container/30 px-3 py-1.5 rounded-md-sm transition-colors disabled:opacity-50"
              >
                {testing === ch.id ? t('channels.testing') : t('channels.test')}
              </button>
            </div>
          </div>
        ))}
        {!loading && channels.length === 0 && (
          <div className="col-span-full text-center py-8 text-body-medium text-md-on-surface-variant">
            {t('channels.empty')}
          </div>
        )}
      </div>
    </div>
  );
}
