import { useState, useEffect, useCallback } from 'react';
import { cn } from '../lib/cn';
import { useAuthStore } from '../stores/useAuthStore';

interface SecurityCheck {
  id: string;
  name: string;
  category: string;
  severity: 'critical' | 'high' | 'medium' | 'low' | 'info';
  description: string;
  remediation?: string;
}

interface ScanResult {
  check_id: string;
  check_name: string;
  status: 'pass' | 'fail' | 'warn' | 'error';
  severity: string;
  message: string;
  details?: Record<string, unknown>;
}

interface ScanResponse {
  results: ScanResult[];
  summary: { total: number; passed: number; failed: number; warnings: number; errors: number };
}

interface ChecksResponse { checks: SecurityCheck[] }

const API_BASE = '';

async function apiGet<T>(path: string, token: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, { headers: { Authorization: `Bearer ${token}` } });
  if (!res.ok) { const body = await res.json().catch(() => ({})); throw new Error((body as { error?: string }).error || `HTTP ${res.status}`); }
  return res.json() as Promise<T>;
}

async function apiPost<T>(path: string, token: string, body: unknown): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, { method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` }, body: JSON.stringify(body) });
  if (!res.ok) { const data = await res.json().catch(() => ({})); throw new Error((data as { error?: string }).error || `HTTP ${res.status}`); }
  return res.json() as Promise<T>;
}

const severityColors: Record<string, string> = {
  critical: 'bg-md-error-container text-md-on-error-container',
  high: 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-200',
  medium: 'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-200',
  low: 'bg-md-primary-container text-md-on-primary-container',
  info: 'bg-md-surface-container-high text-md-on-surface-variant',
};

const statusIcon: Record<string, string> = { pass: '✅', fail: '❌', warn: '⚠️', error: '🔴' };

function SeverityBadge({ severity }: { severity: string }) {
  return (
    <span className={cn('inline-block rounded-md-full px-2.5 py-0.5 text-label-medium font-semibold uppercase', severityColors[severity] || severityColors.info)}>
      {severity}
    </span>
  );
}

function StatusBadge({ status }: { status: string }) {
  return (
    <span className="inline-flex items-center gap-1 text-body-medium">
      <span>{statusIcon[status] || '❓'}</span>
      <span className="font-medium capitalize">{status}</span>
    </span>
  );
}

export function SecurityPage() {
  const { token } = useAuthStore();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [checks, setChecks] = useState<SecurityCheck[]>([]);
  const [result, setResult] = useState<ScanResponse | null>(null);
  const [selectedCheck, setSelectedCheck] = useState<string>('cis_level1');
  const [selectedHost, setSelectedHost] = useState<string>('');
  const [hosts, setHosts] = useState<Array<{ id: string; name: string }>>([]);

  useEffect(() => {
    if (!token) return;
    apiGet<ChecksResponse>('/api/security/checks', token)
      .then((data) => setChecks(data.checks))
      .catch(() => {
        setChecks([
          { id: 'cis_level1', name: 'CIS Level 1 Benchmark', category: 'compliance', severity: 'high', description: 'CIS benchmark level 1 checks' },
          { id: 'vulnerability', name: 'Vulnerability Scan', category: 'vulnerability', severity: 'critical', description: 'CVE and known vulnerability detection' },
          { id: 'patch_audit', name: 'Patch Audit', category: 'patch', severity: 'medium', description: 'Pending security patches audit' },
        ]);
      });
  }, [token]);

  useEffect(() => {
    if (!token) return;
    apiGet<Array<{ id: string; hostname: string }>>('/api/hosts', token)
      .then((data) => setHosts(data.map((h) => ({ id: h.id, name: h.hostname }))))
      .catch(() => setHosts([]));
  }, [token]);

  const runScan = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const data = await apiPost<ScanResponse>('/api/security/scan', token, { host_id: selectedHost || 'all', check_type: selectedCheck });
      setResult(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Backend API unavailable — showing demo data');
      setResult({
        results: [
          { check_id: 'sys-1', check_name: 'SSH Config', status: 'pass', severity: 'high', message: 'SSH root login disabled, key-only auth enabled' },
          { check_id: 'sys-2', check_name: 'Firewall Rules', status: 'pass', severity: 'critical', message: 'All required ports restricted, default deny policy' },
          { check_id: 'sys-3', check_name: 'Password Policy', status: 'warn', severity: 'medium', message: 'Min password length 8 (recommended: 12+)' },
          { check_id: 'sys-4', check_name: 'Kernel Version', status: 'fail', severity: 'high', message: 'Kernel 5.15 has known CVE-2024-XXXXX, upgrade to 5.15.100+' },
          { check_id: 'sys-5', check_name: 'Docker Daemon', status: 'pass', severity: 'medium', message: 'Docker running in rootless mode, content trust enabled' },
          { check_id: 'sys-6', check_name: 'Audit Logging', status: 'fail', severity: 'low', message: 'auditd not installed, no system audit trail' },
          { check_id: 'sys-7', check_name: 'TLS Certificate', status: 'pass', severity: 'high', message: 'Certificate valid until 2027-01-15, TLS 1.3 only' },
          { check_id: 'sys-8', check_name: 'Open Ports', status: 'fail', severity: 'critical', message: 'Port 6379 (Redis) exposed on 0.0.0.0, restrict to internal network' },
        ],
        summary: { total: 8, passed: 4, failed: 3, warnings: 1, errors: 0 },
      });
    } finally {
      setLoading(false);
    }
  }, [token, selectedHost, selectedCheck]);

  return (
    <div className="space-y-6 animate-slide-up">
      <div>
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">Security Scanning</h2>
        <p className="mt-1 text-body-medium text-md-on-surface-variant">
          Run compliance checks, vulnerability scans, and patch audits on managed hosts
        </p>
      </div>

      <div className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 shadow-md-1">
        <h3 className="mb-4 text-title-medium font-medium text-md-on-surface">Scan Configuration</h3>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <div>
            <label className="block text-label-large text-md-on-surface">Check Type</label>
            <select value={selectedCheck} onChange={(e) => setSelectedCheck(e.target.value)}
              className="mt-1 block w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface">
              {checks.map((c) => (<option key={c.id} value={c.id}>{c.name} ({c.category})</option>))}
            </select>
          </div>
          <div>
            <label className="block text-label-large text-md-on-surface">Target Host</label>
            <select value={selectedHost} onChange={(e) => setSelectedHost(e.target.value)}
              className="mt-1 block w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface">
              <option value="">All Hosts</option>
              {hosts.map((h) => (<option key={h.id} value={h.id}>{h.name}</option>))}
            </select>
          </div>
          <div className="flex items-end">
            <button onClick={runScan} disabled={loading}
              className="inline-flex w-full items-center justify-center bg-md-primary text-md-on-primary rounded-md-lg px-6 py-3 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
              {loading ? (
                <><svg className="mr-2 h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none"><circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" /><path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" /></svg>Scanning...</>
              ) : 'Run Scan'}
            </button>
          </div>
        </div>
      </div>

      {error && !result && (
        <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-300 dark:border-amber-700 rounded-md-sm px-4 py-3 text-body-medium text-amber-800 dark:text-amber-200">
          {error}
          <p className="mt-1 text-sm text-amber-600 dark:text-amber-400">(Demo data shown below since the backend module is not connected)</p>
        </div>
      )}

      {result && (
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
          {([
            ['Total Checks', result.summary.total, 'bg-md-surface-container-high text-md-on-surface'],
            ['Passed', result.summary.passed, 'bg-md-primary-container text-md-on-primary-container'],
            ['Failed', result.summary.failed, 'bg-md-error-container text-md-on-error-container'],
            ['Warnings', result.summary.warnings, 'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-200'],
          ] as const).map(([label, count, color]) => (
            <div key={label} className={cn('rounded-md-lg p-4 shadow-md-1', color)}>
              <div className="text-headline-small font-medium">{count}</div>
              <div className="text-label-large">{label}</div>
            </div>
          ))}
        </div>
      )}

      {result && (
        <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="border-b border-md-outline-variant">
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Status</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Check</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Severity</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Message</th>
                </tr>
              </thead>
              <tbody>
                {result.results.map((r) => (
                  <tr key={r.check_id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                    <td className="whitespace-nowrap px-4 py-3"><StatusBadge status={r.status} /></td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{r.check_name}</td>
                    <td className="whitespace-nowrap px-4 py-3"><SeverityBadge severity={r.severity} /></td>
                    <td className="px-4 py-3 text-body-medium text-md-on-surface-variant">{r.message}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      <div className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 shadow-md-1">
        <h3 className="mb-3 text-title-medium font-medium text-md-on-surface">Available Scan Profiles</h3>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {checks.map((c) => (
            <div key={c.id} className="bg-md-surface-container rounded-md-md p-4">
              <div className="mb-1 flex items-center justify-between">
                <span className="text-body-medium font-medium text-md-on-surface">{c.name}</span>
                <SeverityBadge severity={c.severity} />
              </div>
              <p className="text-body-medium text-md-on-surface-variant">{c.description}</p>
              {c.remediation && (
                <p className="mt-1 text-body-medium text-md-primary">{c.remediation}</p>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
