import type {
  ModuleInfo,
  ModuleHealth,
  HealthStatus,
  ModuleConfig,
  Host,
  CreateHostInput,
  AgentSession,
  AgentResponse,
  TopoGraph,
  HostMetrics,
  MetricPoint,
  EscalationTriggerResult,
  FimScanResult,
  BaselineRunResult,
  BaselineReport,
  Runbook,
  RunbookExecution,
  KnowledgeEntry,
  WebhookInfo,
  SchedulerJob,
  FileSyncResult,
  AdvisorSuggestion,
} from './types';

const BASE = '/api';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...init,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(body.error || `HTTP ${res.status}`);
  }
  if (res.status === 204) return undefined as T;
  return res.json();
}

async function requestWithAuth<T>(
  path: string,
  token: string,
  init?: RequestInit,
): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
    ...init,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(body.error || `HTTP ${res.status}`);
  }
  if (res.status === 204) return undefined as T;
  return res.json();
}

export const api = {
  // ── Modules ────────────────────────────────────────────────────────────

  listModules: () => request<ModuleInfo[]>('/modules'),

  getModule: (name: string) =>
    request<ModuleInfo>(`/modules/${encodeURIComponent(name)}`),

  enableModule: (name: string) =>
    request<{ enabled: boolean }>(`/modules/${encodeURIComponent(name)}/enable`, {
      method: 'POST',
    }),

  disableModule: (name: string) =>
    request<{ enabled: boolean }>(`/modules/${encodeURIComponent(name)}/disable`, {
      method: 'POST',
    }),

  getModuleHealth: (name: string) =>
    request<HealthStatus>(`/modules/${encodeURIComponent(name)}/health`),

  getHealthAll: () => request<ModuleHealth[]>('/health'),

  getModuleConfig: (name: string) =>
    request<ModuleConfig>(`/modules/${encodeURIComponent(name)}/config`),

  saveModuleConfig: (name: string, config: ModuleConfig) =>
    request<{ ok: boolean }>(`/modules/${encodeURIComponent(name)}/config`, {
      method: 'PUT',
      body: JSON.stringify(config),
    }),

  // ── Hosts ──────────────────────────────────────────────────────────────

  listHosts: () => request<Host[]>('/hosts'),

  createHost: (input: CreateHostInput) =>
    request<Host>('/hosts', {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  deleteHost: (id: string) =>
    request<void>(`/hosts/${encodeURIComponent(id)}`, {
      method: 'DELETE',
    }),

  // ── Auth ───────────────────────────────────────────────────────────────

  login: (username: string, password: string) =>
    request<{ token: string }>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    }),

  register: (username: string, email: string, password: string) =>
    request<{ id: string; username: string; email: string }>('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ username, email, password }),
    }),

  // ── Agent ──────────────────────────────────────────────────────────────

  createAgentSession: (token: string) =>
    requestWithAuth<AgentSession>('/agent/session', token, {
      method: 'POST',
      body: JSON.stringify({}),
    }),

  sendAgentMessage: (token: string, sessionId: string, message: string) =>
    requestWithAuth<AgentResponse>(`/agent/chat/${sessionId}`, token, {
      method: 'POST',
      body: JSON.stringify({ message }),
    }),

  // ── Vault ─────────────────────────────────────────────────────────────

  getVaultStatus: (token: string) =>
    requestWithAuth<{ unlocked: boolean; has_passphrase: boolean }>('/vault/status', token),

  unlockVault: (token: string, loginPassword: string, passphrase: string) =>
    requestWithAuth<{ status: string }>('/vault/unlock', token, {
      method: 'POST',
      body: JSON.stringify({ login_password: loginPassword, passphrase }),
    }),

  lockVault: (token: string) =>
    requestWithAuth<{ status: string }>('/vault/lock', token, {
      method: 'POST',
      body: JSON.stringify({}),
    }),

  setVaultPassphrase: (token: string, loginPassword: string, passphrase: string, confirm: string) =>
    requestWithAuth<{ status: string }>('/vault/set-passphrase', token, {
      method: 'POST',
      body: JSON.stringify({
        login_password: loginPassword,
        passphrase,
        passphrase_confirm: confirm,
      }),
    }),

  // ── Topology ──────────────────────────────────────────────────────────

  getTopoGraph: (token: string) =>
    requestWithAuth<TopoGraph>('/topo/graph', token),

  discoverTopo: (token: string, hostId?: string) =>
    requestWithAuth<{ nodes: unknown[] }>('/topo/discover', token, {
      method: 'POST',
      body: JSON.stringify(hostId ? { host_id: hostId } : {}),
    }),

  // ── Monitor ───────────────────────────────────────────────────────────

  getMonitorMetrics: (token: string, hostId: string) =>
    requestWithAuth<MetricPoint[]>(`/monitor/metrics/${encodeURIComponent(hostId)}`, token),

  collectMonitorMetrics: (token: string, hostId: string) =>
    requestWithAuth<HostMetrics>('/monitor/collect', token, {
      method: 'POST',
      body: JSON.stringify({ host_id: hostId }),
    }),

  // ── Escalation ────────────────────────────────────────────────────────

  defineEscalationPolicy: (token: string, policy: {
    name: string;
    severity: string;
    escalation_delay_minutes?: number;
    channels?: string[];
  }) =>
    requestWithAuth<{ status: string }>('/escalation/policies', token, {
      method: 'POST',
      body: JSON.stringify(policy),
    }),

  triggerEscalation: (token: string, alertId: string, severity: string, message?: string) =>
    requestWithAuth<EscalationTriggerResult>('/escalation/trigger', token, {
      method: 'POST',
      body: JSON.stringify({ alert_id: alertId, severity, message }),
    }),

  // ── FIM ───────────────────────────────────────────────────────────────

  createFimBaseline: (token: string, hostId: string, paths?: string[]) =>
    requestWithAuth<{ status: string; files_baselined: number }>('/fim/baseline', token, {
      method: 'POST',
      body: JSON.stringify({ host_id: hostId, paths }),
    }),

  fimScan: (token: string, hostId: string) =>
    requestWithAuth<FimScanResult>(`/fim/scan/${encodeURIComponent(hostId)}`, token),

  // ── Baseline ──────────────────────────────────────────────────────────

  runBaselineCheck: (token: string, hostId: string, checkName?: string) =>
    requestWithAuth<BaselineRunResult>(`/baseline/check/${encodeURIComponent(hostId)}`, token, {
      method: 'POST',
      body: JSON.stringify({ check_name: checkName || 'all' }),
    }),

  getBaselineReport: (token: string, hostId: string) =>
    requestWithAuth<BaselineReport>(`/baseline/report/${encodeURIComponent(hostId)}`, token),

  // ── Runbook ───────────────────────────────────────────────────────────

  createRunbook: (token: string, name: string, description: string) =>
    requestWithAuth<Runbook>('/runbook/create', token, {
      method: 'POST',
      body: JSON.stringify({ name, description }),
    }),

  executeRunbook: (token: string, name: string, targetHostId?: string) =>
    requestWithAuth<RunbookExecution>('/runbook/execute', token, {
      method: 'POST',
      body: JSON.stringify({ name, target_host_id: targetHostId }),
    }),

  // ── Knowledge ─────────────────────────────────────────────────────────

  searchKnowledge: (token: string, query: string) =>
    requestWithAuth<{ query: string; results: KnowledgeEntry[] }>('/knowledge/search', token, {
      method: 'POST',
      body: JSON.stringify({ query }),
    }),

  extractKnowledge: (token: string, incidentId: string) =>
    requestWithAuth<KnowledgeEntry>('/knowledge/extract', token, {
      method: 'POST',
      body: JSON.stringify({ incident_id: incidentId }),
    }),

  // ── Config ────────────────────────────────────────────────────────────

  getConfigValue: (token: string, _key: string) =>
    requestWithAuth<{ key: string; value: unknown }>(`/modules/mod-config/config`, token),

  listConfig: (token: string) =>
    requestWithAuth<Record<string, unknown>>('/modules/mod-config/config', token),

  setConfigValue: (token: string, key: string, value: unknown) =>
    requestWithAuth<{ ok: boolean }>(`/modules/mod-config/config`, token, {
      method: 'PUT',
      body: JSON.stringify({ key, value }),
    }),

  // ── Webhook ───────────────────────────────────────────────────────────

  listWebhooks: (token: string) =>
    requestWithAuth<{ webhooks: WebhookInfo[] }>('/modules/mod-webhook/config', token),

  registerWebhook: (token: string, webhook: WebhookInfo) =>
    requestWithAuth<{ status: string }>('/modules/mod-webhook/config', token, {
      method: 'PUT',
      body: JSON.stringify(webhook),
    }),

  // ── Scheduler ─────────────────────────────────────────────────────────

  listSchedulerJobs: (token: string) =>
    requestWithAuth<{ jobs: SchedulerJob[] }>('/modules/mod-scheduler/config', token),

  createSchedulerJob: (token: string, job: { name: string; cron_expr: string; action: string }) =>
    requestWithAuth<{ status: string }>('/modules/mod-scheduler/config', token, {
      method: 'PUT',
      body: JSON.stringify(job),
    }),

  // ── FileSync ──────────────────────────────────────────────────────────

  fileSyncPush: (token: string, hostId: string, filePath: string, content: string) =>
    requestWithAuth<FileSyncResult>('/modules/mod-filesync/config', token, {
      method: 'PUT',
      body: JSON.stringify({ host_id: hostId, file_path: filePath, content }),
    }),

  // ── Advisor ───────────────────────────────────────────────────────────

  listAdvisorSuggestions: (token: string) =>
    requestWithAuth<{ suggestions: AdvisorSuggestion[] }>('/modules/mod-advisor/config', token),

  acknowledgeSuggestion: (token: string, id: string) =>
    requestWithAuth<{ status: string }>('/modules/mod-advisor/config', token, {
      method: 'PUT',
      body: JSON.stringify({ action: 'acknowledge', id }),
    }),

  dismissSuggestion: (token: string, id: string) =>
    requestWithAuth<{ status: string }>('/modules/mod-advisor/config', token, {
      method: 'PUT',
      body: JSON.stringify({ action: 'dismiss', id }),
    }),
};
