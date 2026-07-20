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
      const res = await api.defineEscalationPolicy(token, {
        name: policyName,
        severity,
        escalation_delay_minutes: delay,
        channels: selectedChannels,
      });
      setResult(`策略 "${policyName}" 已创建 (${res.status})`);
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
      setResult(`告警已触发: ${res.status} ${res.policy ? `(策略: ${res.policy})` : ''}`);
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
    <div className="space-y-6">
      <h2 className="text-xl font-semibold text-gray-900">告警升级策略</h2>

      {error && <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>}
      {result && <div className="rounded-md bg-green-50 p-3 text-sm text-green-700">{result}</div>}

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        {/* Define Policy */}
        <div className="rounded-lg border border-gray-200 bg-white p-5 shadow-sm">
          <h3 className="mb-4 text-base font-semibold text-gray-900">定义升级策略</h3>
          <div className="space-y-3">
            <div>
              <label className="block text-sm font-medium text-gray-700">策略名称</label>
              <input value={policyName} onChange={(e) => setPolicyName(e.target.value)} className="mt-1 w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500" placeholder="Critical PagerDuty" />
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-sm font-medium text-gray-700">严重级别</label>
                <select value={severity} onChange={(e) => setSeverity(e.target.value)} className="mt-1 w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm">
                  {SEVERITIES.map((s) => (<option key={s} value={s}>{s}</option>))}
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">延迟（分钟）</label>
                <input type="number" value={delay} onChange={(e) => setDelay(Number(e.target.value))} className="mt-1 w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm" />
              </div>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700">通知渠道</label>
              <div className="mt-1 flex flex-wrap gap-2">
                {CHANNELS.map((ch) => (
                  <button key={ch} onClick={() => toggleChannel(ch)} className={cn('rounded-full border px-3 py-1 text-xs font-medium', selectedChannels.includes(ch) ? 'border-blue-500 bg-blue-50 text-blue-700' : 'border-gray-300 text-gray-600 hover:bg-gray-50')}>
                    {ch}
                  </button>
                ))}
              </div>
            </div>
            <button onClick={handleDefinePolicy} disabled={loading || !policyName} className={cn('w-full rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50')}>
              {loading ? '保存中...' : '保存策略'}
            </button>
          </div>
        </div>

        {/* Trigger Alert */}
        <div className="rounded-lg border border-gray-200 bg-white p-5 shadow-sm">
          <h3 className="mb-4 text-base font-semibold text-gray-900">触发告警</h3>
          <div className="space-y-3">
            <div>
              <label className="block text-sm font-medium text-gray-700">告警 ID</label>
              <input value={alertId} onChange={(e) => setAlertId(e.target.value)} className="mt-1 w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm" placeholder="alert-001" />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700">严重级别</label>
              <select value={alertSeverity} onChange={(e) => setAlertSeverity(e.target.value)} className="mt-1 w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm">
                {SEVERITIES.map((s) => (<option key={s} value={s}>{s}</option>))}
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700">消息</label>
              <textarea value={alertMessage} onChange={(e) => setAlertMessage(e.target.value)} rows={3} className="mt-1 w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500" placeholder="服务宕机描述..." />
            </div>
            <button onClick={handleTrigger} disabled={loading || !alertId} className={cn('w-full rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700 disabled:opacity-50')}>
              {loading ? '触发中...' : '触发告警'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
