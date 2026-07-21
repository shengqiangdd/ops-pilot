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

  // Type-specific config fields
  const [dingtalkUrl, setDingtalkUrl] = useState('');
  const [dingtalkSecret, setDingtalkSecret] = useState('');
  const [wecomUrl, setWecomUrl] = useState('');
  const [smtpHost, setSmtpHost] = useState('');
  const [smtpPort, setSmtpPort] = useState('587');
  const [smtpUser, setSmtpUser] = useState('');
  const [smtpPass, setSmtpPass] = useState('');
  const [smtpFrom, setSmtpFrom] = useState('');
  const [webhookUrl, setWebhookUrl] = useState('');

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

  const buildConfig = () => {
    switch (form.channel_type) {
      case 'dingtalk':
        return { webhook_url: dingtalkUrl, secret: dingtalkSecret || undefined };
      case 'wecom':
        return { webhook_url: wecomUrl };
      case 'email':
        return { smtp_host: smtpHost, smtp_port: parseInt(smtpPort) || 587, smtp_username: smtpUser, smtp_password: smtpPass, from_addr: smtpFrom };
      default:
        return { url: webhookUrl };
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      await api.createNotificationChannel(token!, { ...form, config: buildConfig() });
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
      alert(resp.status === 'ok' ? 'Test sent successfully' : 'Test failed');
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

  const inputCls = 'w-full bg-md-surface-container-highest rounded-md-sm px-4 py-2.5 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface';

  if (loading) return <LoadingState skeleton="list" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.channels')}
        </h2>
        <div className="flex gap-2">
          <button onClick={load} disabled={loading}
            className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors">
            {loading ? '...' : 'Reload'}
          </button>
          <button onClick={() => setShowForm(!showForm)}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all">
            {showForm ? 'Cancel' : '+ Add Channel'}
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {showForm && (
        <form onSubmit={handleSubmit} className="glass-card rounded-md-xl p-5 space-y-4 animate-slide-up">
          <h3 className="text-title-medium font-semibold text-md-on-surface">New Notification Channel</h3>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">Name</label>
              <input type="text" required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })}
                className={inputCls} placeholder="My Channel" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">Type</label>
              <select value={form.channel_type} onChange={(e) => setForm({ ...form, channel_type: e.target.value })} className={inputCls}>
                <option value="webhook">Webhook</option>
                <option value="email">Email (SMTP)</option>
                <option value="dingtalk">DingTalk (钉钉)</option>
                <option value="wecom">WeCom (企业微信)</option>
              </select>
            </div>
          </div>

          {/* Type-specific config */}
          {form.channel_type === 'dingtalk' && (
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <div className="sm:col-span-2">
                <label className="block text-label-large text-md-on-surface mb-1">Webhook URL</label>
                <input type="url" required value={dingtalkUrl} onChange={(e) => setDingtalkUrl(e.target.value)}
                  className={inputCls} placeholder="https://oapi.dingtalk.com/robot/send?access_token=..." />
              </div>
              <div>
                <label className="block text-label-large text-md-on-surface mb-1">Secret (optional)</label>
                <input type="text" value={dingtalkSecret} onChange={(e) => setDingtalkSecret(e.target.value)}
                  className={inputCls} placeholder="SEC..." />
              </div>
            </div>
          )}

          {form.channel_type === 'wecom' && (
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">Webhook URL</label>
              <input type="url" required value={wecomUrl} onChange={(e) => setWecomUrl(e.target.value)}
                className={inputCls} placeholder="https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=..." />
            </div>
          )}

          {form.channel_type === 'email' && (
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <div>
                <label className="block text-label-large text-md-on-surface mb-1">SMTP Host</label>
                <input type="text" required value={smtpHost} onChange={(e) => setSmtpHost(e.target.value)}
                  className={inputCls} placeholder="smtp.gmail.com" />
              </div>
              <div>
                <label className="block text-label-large text-md-on-surface mb-1">Port</label>
                <input type="number" value={smtpPort} onChange={(e) => setSmtpPort(e.target.value)}
                  className={inputCls} placeholder="587" />
              </div>
              <div>
                <label className="block text-label-large text-md-on-surface mb-1">Username</label>
                <input type="text" value={smtpUser} onChange={(e) => setSmtpUser(e.target.value)}
                  className={inputCls} placeholder="user@gmail.com" />
              </div>
              <div>
                <label className="block text-label-large text-md-on-surface mb-1">Password</label>
                <input type="password" value={smtpPass} onChange={(e) => setSmtpPass(e.target.value)}
                  className={inputCls} placeholder="••••••••" />
              </div>
              <div className="sm:col-span-2">
                <label className="block text-label-large text-md-on-surface mb-1">From Address</label>
                <input type="email" value={smtpFrom} onChange={(e) => setSmtpFrom(e.target.value)}
                  className={inputCls} placeholder="opspilot@example.com" />
              </div>
            </div>
          )}

          {form.channel_type === 'webhook' && (
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">Webhook URL</label>
              <input type="url" required value={webhookUrl} onChange={(e) => setWebhookUrl(e.target.value)}
                className={inputCls} placeholder="https://hooks.slack.com/services/..." />
            </div>
          )}

          <div className="flex justify-end gap-2 pt-2">
            <button type="button" onClick={() => setShowForm(false)}
              className="px-4 py-2 text-sm font-medium rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">
              Cancel
            </button>
            <button type="submit" disabled={submitting}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
              {submitting ? 'Creating...' : 'Create Channel'}
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
              <button onClick={() => handleTest(ch.id)} disabled={testing === ch.id}
                className="text-md-primary text-label-large hover:bg-md-primary-container/30 px-3 py-1.5 rounded-md-sm transition-colors disabled:opacity-50">
                {testing === ch.id ? 'Testing...' : 'Test Send'}
              </button>
            </div>
          </div>
        ))}
        {!loading && channels.length === 0 && (
          <div className="col-span-full text-center py-8 text-body-medium text-md-on-surface-variant">
            No notification channels configured. Add one to get started.
          </div>
        )}
      </div>
    </div>
  );
}
