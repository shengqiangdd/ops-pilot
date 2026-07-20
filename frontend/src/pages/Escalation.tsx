import { useCallback, useState } from 'react';
import { api } from '../api/client';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

const SEVERITIES = ['P1', 'P2', 'P3', 'P4'];
const CHANNELS = ['webhook', 'sms', 'email', 'pagerduty', 'chatops'];

export function EscalationPage() {
  const { token } = useAuthStore();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<string | null>(null);

  const [policyName, setPolicyName] = useState('');
  const [severity, setSeverity] = useState('P1');
  const [delay, setDelay] = useState(15);
  const [selectedChannels, setSelectedChannels] = useState<string[]>(['webhook']);

  const [alertId, setAlertId] = useState('');
  const [alertSeverity, setAlertSeverity] = useState('P1');
  const [alertMessage, setAlertMessage] = useState('');

  const handleDefinePolicy = useCallback(async () => {
    if (!token || !policyName) return;
    setLoading(true);
    setError(null);
    try {
      const res = await api.defineEscalationPolicy(token, { name: policyName, severity, escalation_delay_minutes: delay, channels: selectedChannels });
      setResult(`Policy "${policyName}" created (${res.status})`);
      setPolicyName('');
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setLoading(false);
    }
  }, [token, policyName, severity, delay, selectedChannels]);

  const handleTrigger = useCallback(async () => {
    if (!token || !alertId) return;
    setLoading(true);
    setError(null);
    try {
      const res = await api.triggerEscalation(token, alertId, alertSeverity, alertMessage);
      setResult(`Alert triggered: ${res.status} ${res.policy ? `(policy: ${res.policy})` : ''}`);
      setAlertId('');
      setAlertMessage('');
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setLoading(false);
    }
  }, [token, alertId, alertSeverity, alertMessage]);

  const toggleChannel = (ch: string) => {
    setSelectedChannels((prev) => prev.includes(ch) ? prev.filter((c) => c !== ch) : [...prev, ch]);
  };

  return (
    <div className="space-y-6 animate-slide-up">
      <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">Escalation</h2>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}
      {result && <div className="bg-md-primary-container text-md-on-primary-container rounded-md-sm px-4 py-3 text-body-medium">{result}</div>}

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <div className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 shadow-md-1">
          <h3 className="mb-4 text-title-medium font-medium text-md-on-surface">Define Escalation Policy</h3>
          <div className="space-y-3">
            <div>
              <label className="block text-label-large text-md-on-surface">Policy Name</label>
              <input value={policyName} onChange={(e) => setPolicyName(e.target.value)}
                className="mt-1 w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
                placeholder="Critical PagerDuty" />
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-label-large text-md-on-surface">Severity</label>
                <select value={severity} onChange={(e) => setSeverity(e.target.value)}
                  className="mt-1 w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface">
                  {SEVERITIES.map((s) => (<option key={s} value={s}>{s}</option>))}
                </select>
              </div>
              <div>
                <label className="block text-label-large text-md-on-surface">Delay (min)</label>
                <input type="number" value={delay} onChange={(e) => setDelay(Number(e.target.value))}
                  className="mt-1 w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface" />
              </div>
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface">Notification Channels</label>
              <div className="mt-1 flex flex-wrap gap-2">
                {CHANNELS.map((ch) => (
                  <button key={ch} onClick={() => toggleChannel(ch)}
                    className={cn('rounded-md-full border px-3 py-1 text-label-large font-medium transition-colors',
                      selectedChannels.includes(ch)
                        ? 'border-md-primary bg-md-secondary-container text-md-on-secondary-container'
                        : 'border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high')}>
                    {ch}
                  </button>
                ))}
              </div>
            </div>
            <button onClick={handleDefinePolicy} disabled={loading || !policyName}
              className="w-full bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
              {loading ? 'Saving...' : 'Save Policy'}
            </button>
          </div>
        </div>

        <div className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 shadow-md-1">
          <h3 className="mb-4 text-title-medium font-medium text-md-on-surface">Trigger Alert</h3>
          <div className="space-y-3">
            <div>
              <label className="block text-label-large text-md-on-surface">Alert ID</label>
              <input value={alertId} onChange={(e) => setAlertId(e.target.value)}
                className="mt-1 w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface"
                placeholder="alert-001" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface">Severity</label>
              <select value={alertSeverity} onChange={(e) => setAlertSeverity(e.target.value)}
                className="mt-1 w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface">
                {SEVERITIES.map((s) => (<option key={s} value={s}>{s}</option>))}
              </select>
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface">Message</label>
              <textarea value={alertMessage} onChange={(e) => setAlertMessage(e.target.value)} rows={3}
                className="mt-1 w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
                placeholder="Service down description..." />
            </div>
            <button onClick={handleTrigger} disabled={loading || !alertId}
              className="w-full bg-md-error text-md-on-error rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
              {loading ? 'Triggering...' : 'Trigger Alert'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
