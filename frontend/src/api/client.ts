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
  AuditLogEntry,
  UserInfo,
  CreateUserInput,
  AlertRule,
  CreateAlertRuleInput,
  AlertHistoryEntry,
  NotificationChannel,
  CreateChannelInput,
  CMDBService,
  CreateServiceInput,
  ServiceDetail,
  ServiceDependency,
  ConfigVersion,
  CreateConfigInput,
  BatchExecuteRequest,
  BatchExecuteResponse,
  NlQueryResponse,
  DiagnoseRequest,
  DiagnoseResponse,
  TimelineEvent,
  PipelineTemplate,
  CreatePipelineTemplateInput,
  PipelineRun,
  PipelineRunDetail,
  CreatePipelineRunInput,
  Deployment,
  CreateDeploymentInput,
  Job,
  CreateJobInput,
  JobRun,
  JobRunDetail,
  RunDiagnosticsInput,
  DiagnosticReport,
  SystemStatus,
  Report,
  GenerateReportInput,
  ReportSchedule,
  CreateReportScheduleInput,
  Incident,
  IncidentDetail,
  IncidentStats,
  Vulnerability,
  CreateVulnerabilityInput,
  VulnerabilityStats,
  AnalyzePredictionInput,
  PredictionResult,
  RiskItem,
  SLO,
  CreateSloInput,
  BurnRateAlert,
  Playbook,
  CreatePlaybookInput,
  Execution,
  ExecutionDetail,
  RemediationRule,
  CreateRemediationRuleInput,
  RemediationExecution,
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

  listHosts: (token: string) => requestWithAuth<Host[]>('/api/hosts', token),

  createHost: (token: string, input: CreateHostInput) =>
    requestWithAuth<Host>('/api/hosts', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  deleteHost: (token: string, id: string) =>
    requestWithAuth<void>(`/api/hosts/${encodeURIComponent(id)}`, token, {
      method: 'DELETE',
    }),

  // ── Auth ───────────────────────────────────────────────────────────────

  login: (username: string, password: string) =>
    request<{ token: string; role: string }>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    }),

  register: (username: string, email: string, password: string) =>
    request<{ id: string; username: string; email: string; role: string }>('/auth/register', {
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

  // ── Audit ───────────────────────────────────────────────────────────

  listAuditLogs: (token: string, params: Record<string, string>) => {
    const qs = new URLSearchParams(params).toString();
    return requestWithAuth<{
      data: AuditLogEntry[];
      total: number;
      page: number;
      per_page: number;
    }>(`/audit/logs?${qs}`, token);
  },

  getAuditStats: (token: string) =>
    requestWithAuth<{
      total: number;
      by_action: Array<{ action: string; count: number }>;
      by_outcome: Array<{ outcome: string; count: number }>;
    }>('/audit/stats', token),

  // ── Users ───────────────────────────────────────────────────────────

  listUsers: (token: string) =>
    requestWithAuth<UserInfo[]>('/users', token),

  getCurrentUser: (token: string) =>
    requestWithAuth<UserInfo>('/users/me', token),

  createUser: (token: string, input: CreateUserInput) =>
    requestWithAuth<UserInfo>('/users', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  updateUserRole: (token: string, userId: string, role: string) =>
    requestWithAuth<{ status: string }>(`/users/${userId}/role`, token, {
      method: 'PUT',
      body: JSON.stringify({ role }),
    }),

  deleteUser: (token: string, userId: string) =>
    requestWithAuth<void>(`/users/${userId}`, token, {
      method: 'DELETE',
    }),

  // ── Alert Rules ─────────────────────────────────────────────────────

  listAlertRules: (token: string) =>
    requestWithAuth<AlertRule[]>('/alert/rules', token),

  createAlertRule: (token: string, input: CreateAlertRuleInput) =>
    requestWithAuth<AlertRule>('/alert/rules', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  updateAlertRule: (token: string, ruleId: string, updates: Partial<AlertRule>) =>
    requestWithAuth<AlertRule>(`/alert/rules/${ruleId}`, token, {
      method: 'PUT',
      body: JSON.stringify(updates),
    }),

  deleteAlertRule: (token: string, ruleId: string) =>
    requestWithAuth<void>(`/alert/rules/${ruleId}`, token, {
      method: 'DELETE',
    }),

  // ── Alert History ───────────────────────────────────────────────────

  listAlertHistory: (token: string, params?: Record<string, string>) => {
    const qs = params ? new URLSearchParams(params).toString() : '';
    return requestWithAuth<AlertHistoryEntry[]>(`/alert/history${qs ? '?' + qs : ''}`, token);
  },

  // ── Notification Channels ───────────────────────────────────────────

  listNotificationChannels: (token: string) =>
    requestWithAuth<NotificationChannel[]>('/alert/channels', token),

  createNotificationChannel: (token: string, input: CreateChannelInput) =>
    requestWithAuth<NotificationChannel>('/alert/channels', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  testNotificationChannel: (token: string, channelId: string) =>
    requestWithAuth<{ status: string }>(`/alert/channels/${channelId}/test`, token, {
      method: 'POST',
    }),

  // ── CMDB ────────────────────────────────────────────────────────────

  listCMDBServices: (token: string, params?: Record<string, string>) => {
    const qs = params ? new URLSearchParams(params).toString() : '';
    return requestWithAuth<CMDBService[]>(`/cmdb/services${qs ? '?' + qs : ''}`, token);
  },

  createCMDBService: (token: string, input: CreateServiceInput) =>
    requestWithAuth<CMDBService>('/cmdb/services', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  getCMDBServiceDetail: (token: string, serviceId: string) =>
    requestWithAuth<ServiceDetail>(`/cmdb/services/${serviceId}`, token),

  updateCMDBService: (token: string, serviceId: string, updates: Partial<CMDBService>) =>
    requestWithAuth<CMDBService>(`/cmdb/services/${serviceId}`, token, {
      method: 'PUT',
      body: JSON.stringify(updates),
    }),

  deleteCMDBService: (token: string, serviceId: string) =>
    requestWithAuth<void>(`/cmdb/services/${serviceId}`, token, {
      method: 'DELETE',
    }),

  addServiceHost: (token: string, serviceId: string, hostId: string, role?: string) =>
    requestWithAuth<{ id: string }>(`/cmdb/services/${serviceId}/hosts`, token, {
      method: 'POST',
      body: JSON.stringify({ host_id: hostId, role }),
    }),

  removeServiceHost: (token: string, serviceId: string, hostId: string) =>
    requestWithAuth<void>(`/cmdb/services/${serviceId}/hosts/${hostId}`, token, {
      method: 'DELETE',
    }),

  getServiceDependencies: (token: string, serviceId: string) =>
    requestWithAuth<ServiceDependency[]>(`/cmdb/services/${serviceId}/dependencies`, token),

  addServiceDependency: (token: string, serviceId: string, targetServiceId: string, depType?: string) =>
    requestWithAuth<{ id: string }>(`/cmdb/services/${serviceId}/dependencies`, token, {
      method: 'POST',
      body: JSON.stringify({ target_service_id: targetServiceId, dependency_type: depType }),
    }),

  listConfigVersions: (token: string, serviceId?: string) => {
    const qs = serviceId ? `?service_id=${serviceId}` : '';
    return requestWithAuth<ConfigVersion[]>(`/cmdb/configs${qs}`, token);
  },

  createConfigVersion: (token: string, input: CreateConfigInput) =>
    requestWithAuth<ConfigVersion>('/cmdb/configs', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  getConfigVersion: (token: string, configId: string) =>
    requestWithAuth<ConfigVersion>(`/cmdb/configs/${configId}`, token),

  // ── Batch Operations ────────────────────────────────────────────────

  batchExecute: (token: string, input: BatchExecuteRequest) =>
    requestWithAuth<BatchExecuteResponse>('/hosts/batch/execute', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  // ── AI Assistant ─────────────────────────────────────────────────────

  nlQuery: (token: string, query: string) =>
    requestWithAuth<NlQueryResponse>('/agent/nl-query', token, {
      method: 'POST',
      body: JSON.stringify({ query }),
    }),

  diagnose: (token: string, input: DiagnoseRequest) =>
    requestWithAuth<DiagnoseResponse>('/agent/diagnose', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  // ── Timeline ────────────────────────────────────────────────────────

  getTimelineEvents: (token: string, params?: Record<string, string>) => {
    const qs = params ? new URLSearchParams(params).toString() : '';
    return requestWithAuth<TimelineEvent[]>(`/timeline/events${qs ? '?' + qs : ''}`, token);
  },

  // ── CI/CD ───────────────────────────────────────────────────────────

  listCICDTemplates: (token: string) =>
    requestWithAuth<PipelineTemplate[]>('/cicd/templates', token),

  createCICDTemplate: (token: string, input: CreatePipelineTemplateInput) =>
    requestWithAuth<PipelineTemplate>('/cicd/templates', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  deleteCICDTemplate: (token: string, templateId: string) =>
    requestWithAuth<void>(`/cicd/templates/${templateId}`, token, {
      method: 'DELETE',
    }),

  listCICDRuns: (token: string) =>
    requestWithAuth<PipelineRun[]>('/cicd/runs', token),

  createCICDRun: (token: string, input: CreatePipelineRunInput) =>
    requestWithAuth<PipelineRun>('/cicd/runs', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  getCICDRunDetail: (token: string, runId: string) =>
    requestWithAuth<PipelineRunDetail>(`/cicd/runs/${runId}`, token),

  cancelCICDRun: (token: string, runId: string) =>
    requestWithAuth<{ status: string }>(`/cicd/runs/${runId}/cancel`, token, {
      method: 'POST',
    }),

  listCICDDeployments: (token: string) =>
    requestWithAuth<Deployment[]>('/cicd/deployments', token),

  createCICDDeployment: (token: string, input: CreateDeploymentInput) =>
    requestWithAuth<Deployment>('/cicd/deployments', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  rollbackCICDDeployment: (token: string, deploymentId: string) =>
    requestWithAuth<Deployment>(`/cicd/deployments/${deploymentId}/rollback`, token, {
      method: 'PUT',
    }),

  // ── Jobs ─────────────────────────────────────────────────────────────

  listJobs: (token: string) =>
    requestWithAuth<Job[]>('/jobs', token),

  createJob: (token: string, input: CreateJobInput) =>
    requestWithAuth<Job>('/jobs', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  getJob: (token: string, jobId: string) =>
    requestWithAuth<Job>(`/jobs/${jobId}`, token),

  deleteJob: (token: string, jobId: string) =>
    requestWithAuth<void>(`/jobs/${jobId}`, token, {
      method: 'DELETE',
    }),

  executeJob: (token: string, jobId: string) =>
    requestWithAuth<JobRun>(`/jobs/${jobId}/execute`, token, {
      method: 'POST',
    }),

  listJobRuns: (token: string, jobId: string) =>
    requestWithAuth<JobRun[]>(`/jobs/${jobId}/runs`, token),

  getJobRunDetail: (token: string, runId: string) =>
    requestWithAuth<JobRunDetail>(`/jobs/runs/${runId}`, token),

  // ── Diagnostics ────────────────────────────────────────────────────

  runDiagnostics: (token: string, input: RunDiagnosticsInput) =>
    requestWithAuth<DiagnosticReport>('/diagnostics/run', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  getDiagnosticsHistory: (token: string, params?: Record<string, string>) => {
    const qs = params ? new URLSearchParams(params).toString() : '';
    return requestWithAuth<DiagnosticReport[]>(`/diagnostics/history${qs ? '?' + qs : ''}`, token);
  },

  getDiagnosticsDetail: (token: string, diagId: string) =>
    requestWithAuth<DiagnosticReport>(`/diagnostics/${diagId}`, token),

  getDiagnosticsStatus: (token: string) =>
    requestWithAuth<SystemStatus>('/diagnostics/status', token),

  // ── Reports ────────────────────────────────────────────────────────

  generateReport: (token: string, input: GenerateReportInput) =>
    requestWithAuth<Report>('/reports', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  listReports: (token: string, params?: Record<string, string>) => {
    const qs = params ? new URLSearchParams(params).toString() : '';
    return requestWithAuth<Report[]>(`/reports${qs ? '?' + qs : ''}`, token);
  },

  getReport: (token: string, reportId: string) =>
    requestWithAuth<Report>(`/reports/${reportId}`, token),

  exportReport: (token: string, reportId: string) =>
    requestWithAuth<string>(`/reports/${reportId}/export`, token),

  listReportSchedules: (token: string) =>
    requestWithAuth<ReportSchedule[]>('/reports/schedule', token),

  createReportSchedule: (token: string, input: CreateReportScheduleInput) =>
    requestWithAuth<ReportSchedule>('/reports/schedule', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  // ── Incidents ──────────────────────────────────────────────────────

  listIncidents: (token: string, params?: Record<string, string>) => {
    const qs = params ? new URLSearchParams(params).toString() : '';
    return requestWithAuth<Incident[]>(`/incidents${qs ? '?' + qs : ''}`, token);
  },

  getIncident: (token: string, incidentId: string) =>
    requestWithAuth<IncidentDetail>(`/incidents/${incidentId}`, token),

  updateIncident: (token: string, incidentId: string, updates: { status?: string; assigned_to?: string }) =>
    requestWithAuth<Incident>(`/incidents/${incidentId}`, token, {
      method: 'PUT',
      body: JSON.stringify(updates),
    }),

  assignIncident: (token: string, incidentId: string, input: { assigned_to: string }) =>
    requestWithAuth<{ status: string }>(`/incidents/${incidentId}/assign`, token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  getIncidentStats: (token: string) =>
    requestWithAuth<IncidentStats>('/incidents/stats', token),

  // ── Vulnerabilities ────────────────────────────────────────────────

  listVulnerabilities: (token: string, params?: Record<string, string>) => {
    const qs = params ? new URLSearchParams(params).toString() : '';
    return requestWithAuth<Vulnerability[]>(`/vulnerabilities${qs ? '?' + qs : ''}`, token);
  },

  createVulnerability: (token: string, input: CreateVulnerabilityInput) =>
    requestWithAuth<Vulnerability>('/vulnerabilities', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  getVulnerability: (token: string, vulnId: string) =>
    requestWithAuth<Vulnerability>(`/vulnerabilities/${vulnId}`, token),

  updateVulnerability: (token: string, vulnId: string, updates: { status?: string; assigned_to?: string; notes?: string }) =>
    requestWithAuth<Vulnerability>(`/vulnerabilities/${vulnId}`, token, {
      method: 'PUT',
      body: JSON.stringify(updates),
    }),

  deleteVulnerability: (token: string, vulnId: string) =>
    requestWithAuth<void>(`/vulnerabilities/${vulnId}`, token, {
      method: 'DELETE',
    }),

  verifyVulnerability: (token: string, vulnId: string) =>
    requestWithAuth<{ status: string }>(`/vulnerabilities/${vulnId}/verify`, token, {
      method: 'POST',
    }),

  getVulnerabilityStats: (token: string) =>
    requestWithAuth<VulnerabilityStats>('/vulnerabilities/stats', token),

  scanVulnerabilities: (token: string) =>
    requestWithAuth<{ status: string; new_vulnerabilities: number }>('/vulnerabilities/scan', token, {
      method: 'POST',
    }),

  // ── Predictions ────────────────────────────────────────────────────

  analyzePrediction: (token: string, input: AnalyzePredictionInput) =>
    requestWithAuth<PredictionResult>('/predictions/analyze', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  batchPrediction: (token: string, params?: { forecast_hours?: number }) =>
    requestWithAuth<PredictionResult[]>('/predictions/batch', token, {
      method: 'POST',
      body: JSON.stringify(params || {}),
    }),

  getPredictionRisks: (token: string) =>
    requestWithAuth<RiskItem[]>('/predictions/risks', token),

  // ── SLOs ───────────────────────────────────────────────────────────

  listSlos: (token: string, params?: Record<string, string>) => {
    const qs = params ? new URLSearchParams(params).toString() : '';
    return requestWithAuth<SLO[]>(`/slos${qs ? '?' + qs : ''}`, token);
  },

  createSlo: (token: string, input: CreateSloInput) =>
    requestWithAuth<SLO>('/slos', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  getSlo: (token: string, sloId: string) =>
    requestWithAuth<{ slo: SLO; events: unknown[] }>(`/slos/${sloId}`, token),

  deleteSlo: (token: string, sloId: string) =>
    requestWithAuth<void>(`/slos/${sloId}`, token, {
      method: 'DELETE',
    }),

  evaluateSlo: (token: string, sloId: string) =>
    requestWithAuth<SLO>(`/slos/${sloId}/evaluate`, token, {
      method: 'POST',
    }),

  getBurnRateAlerts: (token: string) =>
    requestWithAuth<BurnRateAlert[]>('/slos/burn-rate', token),

  // ── SOAR ───────────────────────────────────────────────────────────

  listPlaybooks: (token: string) =>
    requestWithAuth<Playbook[]>('/soar/playbooks', token),

  createPlaybook: (token: string, input: CreatePlaybookInput) =>
    requestWithAuth<Playbook>('/soar/playbooks', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  getPlaybook: (token: string, playbookId: string) =>
    requestWithAuth<Playbook>(`/soar/playbooks/${playbookId}`, token),

  deletePlaybook: (token: string, playbookId: string) =>
    requestWithAuth<void>(`/soar/playbooks/${playbookId}`, token, {
      method: 'DELETE',
    }),

  executePlaybook: (token: string, playbookId: string) =>
    requestWithAuth<Execution>(`/soar/playbooks/${playbookId}/execute`, token, {
      method: 'POST',
    }),

  listSoarExecutions: (token: string, params?: Record<string, string>) => {
    const qs = params ? new URLSearchParams(params).toString() : '';
    return requestWithAuth<Execution[]>(`/soar/executions${qs ? '?' + qs : ''}`, token);
  },

  getSoarExecution: (token: string, execId: string) =>
    requestWithAuth<ExecutionDetail>(`/soar/executions/${execId}`, token),

  // ── Remediation ─────────────────────────────────────────────────────

  listRemediationRules: (token: string) =>
    requestWithAuth<RemediationRule[]>('/remediation/rules', token),

  createRemediationRule: (token: string, input: CreateRemediationRuleInput) =>
    requestWithAuth<RemediationRule>('/remediation/rules', token, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  deleteRemediationRule: (token: string, ruleId: string) =>
    requestWithAuth<void>(`/remediation/rules/${ruleId}`, token, {
      method: 'DELETE',
    }),

  testRemediationRule: (token: string, ruleId: string) =>
    requestWithAuth<{ status: string }>(`/remediation/rules/${ruleId}/test`, token, {
      method: 'POST',
    }),

  listRemediationExecutions: (token: string, params?: Record<string, string>) => {
    const qs = params ? new URLSearchParams(params).toString() : '';
    return requestWithAuth<RemediationExecution[]>(`/remediation/executions${qs ? '?' + qs : ''}`, token);
  },
};
