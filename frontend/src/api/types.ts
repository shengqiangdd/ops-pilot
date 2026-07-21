/** 模块基本信息 */
export interface ModuleInfo {
  /** 模块名称，如 "mod-core::ssh" */
  name: string;
  /** 语义化版本号 */
  version: string;
  /** 模块功能描述 */
  description: string;
  /** 是否启用 */
  enabled: boolean;
}

/**
 * 健康检查状态联合类型
 *
 * - `Healthy` — 模块运行正常
 * - `Degraded` — 模块部分降级，附带原因说明
 * - `Unhealthy` — 模块不可用，附带原因说明
 */
export type HealthStatus =
  | { Healthy: null }
  | { Degraded: { reason: string } }
  | { Unhealthy: { reason: string } };

/** 模块健康信息，包含名称、状态和启用状态 */
export interface ModuleHealth {
  /** 模块名称 */
  name: string;
  /** 健康检查状态 */
  status: HealthStatus;
  /** 模块是否启用 */
  enabled: boolean;
}

/** 模块自定义配置（键值对） */
export interface ModuleConfig {
  [key: string]: unknown;
}

// ── Host types ──────────────────────────────────────────────────────────

/** 主机状态枚举 */
export type HostStatus = 'online' | 'offline' | 'unknown' | 'maintenance';

/** 主机配置信息 */
export interface Host {
  /** 主机唯一标识 */
  id: string;
  /** 主机名称 */
  name: string;
  /** 主机地址（IP 或域名） */
  address: string;
  /** SSH 端口号 */
  port: number;
  /** SSH 登录用户名 */
  username: string;
  /** 认证方式（如 "key" 或 "password"） */
  auth_method: string;
  /** 主机当前状态 */
  status: HostStatus;
  /** 创建时间（ISO 8601） */
  created_at: string;
  /** 最后更新时间（ISO 8601） */
  updated_at: string;
}

/** 创建主机的请求参数 */
export interface CreateHostInput {
  name: string;
  address: string;
  port?: number;
  username: string;
  auth_method: string;
  password?: string;
  private_key?: string;
}

// ── Agent types ─────────────────────────────────────────────────────────

export interface AgentSession {
  session_id: string;
}

export interface AgentResponse {
  session_id: string;
  content: string;
  turns: AgentTurn[];
  truncated: boolean;
}

export interface AgentTurn {
  turn: number;
  action: string;
  result: string;
}

// ── Topology types ──────────────────────────────────────────────────────

export interface TopoNode {
  id: string;
  label: string;
  kind: 'Host' | 'Service' | 'Container' | 'LoadBalancer' | 'Database' | 'External';
  properties: Record<string, string>;
}

export interface TopoEdge {
  source: string;
  target: string;
  label: string;
  protocol?: string;
  port?: number;
}

export interface TopoGraph {
  nodes: TopoNode[];
  edges: TopoEdge[];
}

// ── Monitor types ───────────────────────────────────────────────────────

export interface MetricPoint {
  timestamp: string;
  host_id: string;
  metric_type: string;
  value: number;
  unit: string;
}

export interface HostMetrics {
  host_id: string;
  timestamp: string;
  cpu_percent: number;
  memory_percent: number;
  memory_total_mb: number;
  memory_used_mb: number;
  disk_percent: number;
  disk_total_gb: number;
  disk_used_gb: number;
  network_in_bytes: number;
  network_out_bytes: number;
  load_1: number;
  load_5: number;
  load_15: number;
}

// ── Escalation types ────────────────────────────────────────────────────

export interface EscalationPolicy {
  name: string;
  severity: string;
  delay_minutes: number;
  channels: string[];
}

export interface EscalationTriggerResult {
  status: string;
  alert_id: string;
  policy?: string;
  channels?: string[];
  delay_minutes?: number;
}

// ── FIM types ───────────────────────────────────────────────────────────

export interface FimChange {
  path: string;
  status: 'modified' | 'deleted' | 'added';
  old_hash?: string;
  new_hash?: string;
  hash?: string;
}

export interface FimScanResult {
  host_id: string;
  changes: FimChange[];
  total_files: number;
}

// ── Baseline types ──────────────────────────────────────────────────────

export interface BaselineCheckResult {
  name: string;
  category: string;
  status: 'Pass' | 'Fail' | 'Warn' | 'Skip' | 'Info';
  message: string;
  remediation?: string;
}

export interface BaselineRunResult {
  host_id: string;
  results: BaselineCheckResult[];
  score: number;
}

export interface BaselineReport {
  host_id: string;
  results: BaselineCheckResult[];
  score: number;
}

// ── Runbook types ───────────────────────────────────────────────────────

export interface RunbookStep {
  id: string;
  name: string;
  command: string;
  requires_approval: boolean;
  timeout_seconds: number;
}

export interface Runbook {
  name: string;
  description: string;
  steps: RunbookStep[];
}

export interface RunbookExecution {
  runbook_name: string;
  host_id: string;
  started_at: string;
  finished_at: string;
  success: boolean;
  steps: Array<{
    step_id: string;
    status: string;
    output: string;
    duration_ms: number;
  }>;
}

// ── Knowledge types ─────────────────────────────────────────────────────

export interface KnowledgeEntry {
  id: string;
  incident_id: string;
  title: string;
  root_cause: string;
  resolution: string;
  tags: string[];
  created_at: string;
}

// ── Config types ────────────────────────────────────────────────────────

export interface ConfigEntry {
  key: string;
  value: unknown;
  description?: string;
}

// ── Webhook types ───────────────────────────────────────────────────────

export interface WebhookInfo {
  name: string;
  url: string;
  secret?: string;
  retry_count: number;
}

// ── Scheduler types ─────────────────────────────────────────────────────

export interface SchedulerJob {
  name: string;
  cron_expr: string;
  action: string;
  enabled: boolean;
  last_run_at?: string;
  next_run_at?: string;
}

// ── FileSync types ──────────────────────────────────────────────────────

export interface FileSyncResult {
  status: string;
  host_id: string;
  file_path?: string;
  size?: number;
}

// ── Advisor types ───────────────────────────────────────────────────────

export interface AdvisorSuggestion {
  id: string;
  severity: string;
  category: string;
  title: string;
  description: string;
  suggested_action: string;
  created_at: string;
  acknowledged: boolean;
  dismissed: boolean;
}

// ── Audit types ────────────────────────────────────────────────────────

export interface AuditLogEntry {
  id: string;
  user: string;
  action: string;
  resource: string;
  outcome: string;
  created_at: string;
}

export interface AuditLogResponse {
  data: AuditLogEntry[];
  total: number;
  page: number;
  per_page: number;
}

// ── User management types ─────────────────────────────────────────────

export interface UserInfo {
  id: string;
  username: string;
  email: string;
  role: string;
  created_at: string;
}

export interface CreateUserInput {
  username: string;
  email: string;
  password: string;
  role?: string;
}

// ── Alert types ────────────────────────────────────────────────────────

export interface AlertRule {
  id: string;
  name: string;
  metric: string;
  condition: string;
  threshold: number;
  severity: string;
  silence_minutes: number;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateAlertRuleInput {
  name: string;
  metric: string;
  condition: string;
  threshold: number;
  severity: string;
  silence_minutes?: number;
  channel_ids?: string[];
}

export interface AlertHistoryEntry {
  id: string;
  rule_id: string;
  rule_name: string;
  severity: string;
  message: string;
  status: string;
  triggered_at: string;
  acknowledged_at: string | null;
}

export interface NotificationChannel {
  id: string;
  name: string;
  channel_type: string;
  config: string;
  enabled: boolean;
  created_at: string;
}

export interface CreateChannelInput {
  name: string;
  channel_type: string;
  config: Record<string, unknown>;
}

// ── CMDB types ─────────────────────────────────────────────────────────

export interface CMDBService {
  id: string;
  name: string;
  version: string;
  description: string;
  owner: string;
  status: string;
  created_at: string;
  updated_at: string;
}

export interface CreateServiceInput {
  name: string;
  version?: string;
  description?: string;
  owner?: string;
  status?: string;
}

export interface ServiceHost {
  id: string;
  service_id: string;
  host_id: string;
  role: string;
  created_at: string;
}

export interface ServiceDependency {
  id: string;
  source_service_id: string;
  target_service_id: string;
  dependency_type: string;
  description: string;
  created_at: string;
}

export interface ServiceDetail {
  service: CMDBService;
  hosts: ServiceHost[];
  dependencies: ServiceDependency[];
}

export interface ConfigVersion {
  id: string;
  service_id: string;
  config_json: string;
  version: number;
  changed_by: string;
  change_note: string;
  created_at: string;
}

export interface CreateConfigInput {
  service_id: string;
  config_json: string;
  changed_by?: string;
  change_note?: string;
}

// ── Batch execution types ──────────────────────────────────────────────

export interface BatchExecuteRequest {
  host_ids: string[];
  command: string;
  timeout?: number;
}

export interface BatchExecuteResult {
  host_id: string;
  host_name: string;
  success: boolean;
  stdout: string;
  stderr: string;
  exit_code: number;
  duration_ms: number;
}

export interface BatchExecuteResponse {
  results: BatchExecuteResult[];
  total: number;
  succeeded: number;
  failed: number;
}

// ── AI Assistant types ─────────────────────────────────────────────────

export interface NlQueryRequest {
  query: string;
}

export interface NlQueryResponse {
  query: string;
  parsed_intent: string;
  results: Array<Record<string, unknown>>;
  summary: string;
}

export interface DiagnoseRequest {
  host_id?: string;
  issue_description: string;
}

export interface DiagnoseResponse {
  issue: string;
  severity: string;
  possible_causes: string[];
  recommended_actions: string[];
  related_knowledge: Array<Record<string, unknown>>;
}

// ── Timeline types ─────────────────────────────────────────────────────

export interface TimelineEvent {
  id: string;
  timestamp: string;
  type: string;
  severity: string;
  title: string;
  description: string;
  source: string;
}

// ── CI/CD types ────────────────────────────────────────────────────────

export interface PipelineTemplate {
  id: string;
  name: string;
  description: string;
  stages_json: string;
  created_at: string;
  updated_at: string;
}

export interface CreatePipelineTemplateInput {
  name: string;
  description?: string;
  stages_json?: string;
}

export interface PipelineRun {
  id: string;
  template_id: string;
  name: string;
  status: string;
  triggered_by: string;
  branch: string;
  commit_sha: string;
  started_at: string | null;
  finished_at: string | null;
  duration_ms: number | null;
  created_at: string;
}

export interface PipelineStageRun {
  id: string;
  run_id: string;
  stage_name: string;
  status: string;
  log: string;
  started_at: string | null;
  finished_at: string | null;
}

export interface PipelineRunDetail {
  run: PipelineRun;
  stages: PipelineStageRun[];
}

export interface CreatePipelineRunInput {
  template_id: string;
  name?: string;
  branch?: string;
  commit_sha?: string;
}

export interface Deployment {
  id: string;
  name: string;
  service_id: string;
  environment: string;
  strategy: string;
  status: string;
  version: string;
  config_json: string;
  started_at: string | null;
  finished_at: string | null;
  created_at: string;
}

export interface CreateDeploymentInput {
  name: string;
  service_id?: string;
  environment?: string;
  strategy?: string;
  version?: string;
}

// ── Jobs types ─────────────────────────────────────────────────────────

export interface Job {
  id: string;
  name: string;
  description: string;
  steps_json: string;
  retry_policy: string;
  timeout_seconds: number;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateJobInput {
  name: string;
  description?: string;
  steps_json: string;
  retry_policy?: string;
  timeout_seconds?: number;
}

export interface JobRun {
  id: string;
  job_id: string;
  status: string;
  triggered_by: string;
  started_at: string | null;
  finished_at: string | null;
  duration_ms: number | null;
  error_message: string | null;
  created_at: string;
}

export interface JobStepRun {
  id: string;
  run_id: string;
  step_name: string;
  status: string;
  output: string;
  error: string | null;
  started_at: string | null;
  finished_at: string | null;
}

export interface JobRunDetail {
  run: JobRun;
  steps: JobStepRun[];
}

// ── Diagnostics types ──────────────────────────────────────────────────

export interface RunDiagnosticsInput {
  host_id?: string;
  checks?: string[];
}

export interface DiagnosticItem {
  check_name: string;
  status: string;
  value: string;
  threshold: string | null;
  message: string;
  suggestion: string;
}

export interface DiagnosticCategory {
  name: string;
  status: string;
  score: number;
  items: DiagnosticItem[];
}

export interface DiagnosticReport {
  id: string;
  host_id: string;
  timestamp: string;
  overall_status: string;
  overall_score: number;
  categories: DiagnosticCategory[];
}

export interface SystemStatus {
  total_hosts: number;
  online_hosts: number;
  offline_hosts: number;
  total_services: number;
  active_services: number;
  total_alert_rules: number;
  active_alert_rules: number;
  recent_alerts: number;
  overall_health_score: number;
}

// ── Reports types ──────────────────────────────────────────────────────

export interface Report {
  id: string;
  report_type: string;
  title: string;
  summary: string;
  content_html: string;
  host_ids: string;
  sections: string;
  created_at: string;
}

export interface GenerateReportInput {
  report_type: string;
  host_ids?: string[];
  include_sections?: string[];
}

export interface ReportSchedule {
  id: string;
  enabled: boolean;
  report_type: string;
  recipients: string;
  day_of_week: number | null;
  day_of_month: number | null;
  last_generated_at: string | null;
  created_at: string;
}

export interface CreateReportScheduleInput {
  enabled: boolean;
  report_type: string;
  recipients: string[];
  day_of_week?: number;
  day_of_month?: number;
}

// ── Incidents types ────────────────────────────────────────────────────

export interface Incident {
  id: string;
  name: string;
  status: string;
  severity: string;
  host_id: string;
  first_seen: string;
  last_seen: string;
  alert_count: number;
  summary: string;
  assigned_to: string;
  created_at: string;
}

export interface IncidentAlert {
  id: string;
  incident_id: string;
  alert_id: string;
  alert_message: string;
  alert_severity: string;
  triggered_at: string;
}

export interface IncidentDetail {
  incident: Incident;
  alerts: IncidentAlert[];
}

export interface IncidentStats {
  total: number;
  open: number;
  acknowledged: number;
  resolved: number;
}

// ── Vulnerabilities types ──────────────────────────────────────────────

export interface Vulnerability {
  id: string;
  cve_id: string;
  title: string;
  description: string;
  severity: string;
  cvss_score: number;
  affected_host: string;
  affected_service: string;
  status: string;
  discovered_at: string;
  assigned_to: string;
  fixed_at: string | null;
  notes: string;
  created_at: string;
}

export interface CreateVulnerabilityInput {
  cve_id: string;
  title: string;
  description?: string;
  severity: string;
  cvss_score?: number;
  affected_host?: string;
  affected_service?: string;
  notes?: string;
}

export interface VulnerabilityStats {
  total: number;
  critical: number;
  high: number;
  medium: number;
  low: number;
  open: number;
  in_progress: number;
  fixed: number;
}

// ── Predictions types ──────────────────────────────────────────────────

export interface AnalyzePredictionInput {
  host_id: string;
  metric_type: string;
  forecast_hours?: number;
  threshold?: number;
}

export interface PredictionDataPoint {
  timestamp: string;
  actual: number | null;
  predicted: number | null;
}

export interface PredictionResult {
  host_id: string;
  metric_type: string;
  current_value: number;
  predicted_value: number;
  trend: string;
  confidence: number;
  risk_level: string;
  estimated_time_to_threshold_hours: number | null;
  data_points: PredictionDataPoint[];
}

export interface RiskItem {
  host_id: string;
  host_name: string;
  metric_type: string;
  current_value: number;
  predicted_value: number;
  threshold: number;
  risk_level: string;
  estimated_time_hours: number;
  suggestion: string;
}

// ── SLO types ──────────────────────────────────────────────────────────

export interface SLO {
  id: string;
  name: string;
  description: string;
  service_id: string;
  sli_type: string;
  target_percentage: number;
  window_days: number;
  current_sli: number;
  error_budget_remaining: number;
  status: string;
  created_at: string;
}

export interface CreateSloInput {
  name: string;
  description?: string;
  service_id?: string;
  sli_type: string;
  target_percentage: number;
  window_days?: number;
}

export interface BurnRateAlert {
  slo_id: string;
  slo_name: string;
  burn_rate: number;
  error_budget_remaining: number;
  estimated_breach_hours: number;
  severity: string;
  suggestion: string;
}

// ── SOAR types ─────────────────────────────────────────────────────────

export interface Playbook {
  id: string;
  name: string;
  description: string;
  trigger_type: string;
  trigger_conditions_json: string;
  steps_json: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreatePlaybookInput {
  name: string;
  description?: string;
  trigger_type: string;
  trigger_conditions_json?: string;
  steps_json: string;
}

export interface Execution {
  id: string;
  playbook_id: string;
  trigger_source: string;
  trigger_id: string;
  status: string;
  result_json: string;
  started_at: string | null;
  finished_at: string | null;
}

export interface ExecutionStep {
  step_type: string;
  status: string;
  message: string;
  duration_ms: number | null;
}

export interface ExecutionDetail {
  execution: Execution;
  playbook_name: string;
  steps: ExecutionStep[];
}

// ── Remediation types ──────────────────────────────────────────────────

export interface RemediationRule {
  id: string;
  name: string;
  trigger_type: string;
  trigger_condition_json: string;
  actions_json: string;
  cooldown_minutes: number;
  max_retries: number;
  enabled: boolean;
  created_at: string;
}

export interface CreateRemediationRuleInput {
  name: string;
  trigger_type: string;
  trigger_condition_json?: string;
  actions_json: string;
  cooldown_minutes?: number;
  max_retries?: number;
}

export interface RemediationExecution {
  id: string;
  rule_id: string;
  trigger_id: string;
  trigger_type: string;
  status: string;
  result_json: string;
  started_at: string | null;
  finished_at: string | null;
}

// ── Secrets Scan types ─────────────────────────────────────────────────

export interface ScanResult {
  id: string;
  host_id: string;
  file_path: string;
  scan_type: string;
  severity: string;
  line_number: number;
  snippet: string;
  finding: string;
  suggestion: string;
  status: string;
  discovered_at: string;
}

export interface ScanStats {
  total: number;
  by_type: Array<{ scan_type: string; count: number }>;
  by_severity: Array<{ severity: string; count: number }>;
}

export interface ScanSecretsInput {
  host_ids?: string[];
  scan_types?: string[];
}

export interface UpdateSecretsResultInput {
  status: string;
}

// ── Compliance types ───────────────────────────────────────────────────

export interface ComplianceFramework {
  id: string;
  name: string;
  version: string;
  description: string;
}

export interface ComplianceOverview {
  total_controls: number;
  passed: number;
  failed: number;
  not_applicable: number;
  pass_rate: number;
  by_category: Array<{ category: string; total: number; passed: number; failed: number }>;
}

export interface ComplianceReport {
  framework: ComplianceFramework;
  overview: ComplianceOverview;
  controls: Array<{ id: string; control_id: string; title: string; description: string; category: string; severity: string }>;
}

export interface ScanComplianceInput {
  framework_id?: string;
  host_id?: string;
}

// ── Threats types ──────────────────────────────────────────────────────

export interface ThreatOverview {
  total_indicators: number;
  affected_assets: number;
  critical_count: number;
  high_count: number;
  medium_count: number;
  low_count: number;
  today_new: number;
}

export interface ThreatIndicator {
  id: string;
  feed_id: string;
  indicator_type: string;
  indicator_value: string;
  severity: string;
  title: string;
  description: string;
  first_seen: string;
  last_seen: string;
}

export interface AffectedAsset {
  id: string;
  host_id: string;
  host_name: string;
  indicator_type: string;
  indicator_value: string;
  severity: string;
  threat_title: string;
  risk_level: string;
  suggestion: string;
}

// ── Change Analysis types ──────────────────────────────────────────────

export interface ChangeEvent {
  id: string;
  host_id: string;
  change_type: string;
  source: string;
  description: string;
  content_diff: string;
  risk_score: number;
  risk_factors_json: string;
  status: string;
  proposed_by: string;
  reviewed_by: string;
  created_at: string;
  reviewed_at: string | null;
}

export interface CreateChangeEventInput {
  host_id: string;
  change_type: string;
  source: string;
  description: string;
  content_diff?: string;
  proposed_by?: string;
}

export interface ChangeStats {
  total: number;
  pending: number;
  approved: number;
  rejected: number;
  high_risk: number;
}

// ── Log Intelligence types ─────────────────────────────────────────────

export interface LogSource {
  id: string;
  host_id: string;
  source_name: string;
  log_path: string;
  source_type: string;
  enabled: boolean;
}

export interface LogPattern {
  id: string;
  host_id: string;
  pattern: string;
  pattern_type: string;
  count: number;
  first_seen: string;
  last_seen: string;
  severity: string;
}

export interface LogAnomaly {
  id: string;
  host_id: string;
  source_id: string;
  anomaly_type: string;
  description: string;
  severity: string;
  detected_at: string;
  status: string;
}

export interface LogIntelStats {
  total_sources: number;
  total_patterns: number;
  total_anomalies: number;
  open_anomalies: number;
}

// ── On-Call types ──────────────────────────────────────────────────────

export interface OnCallSchedule {
  id: string;
  name: string;
  description: string;
  timezone: string;
  rotation_type: string;
  starts_at: string;
  ends_at: string;
  enabled: boolean;
  created_at: string;
}

export interface OnCallShift {
  id: string;
  schedule_id: string;
  user_id: string;
  start_time: string;
  end_time: string;
  role: string;
}

export interface OnCallOverride {
  id: string;
  schedule_id: string;
  user_id: string;
  date: string;
  reason: string;
}

export interface OnCallEscalation {
  id: string;
  alert_id: string;
  shift_id: string;
  notified_at: string;
  acknowledged_at: string | null;
  status: string;
}

export interface CreateOnCallScheduleInput {
  name: string;
  description?: string;
  timezone?: string;
  rotation_type?: string;
}

// ── Chaos Engineering types ────────────────────────────────────────────

export interface ChaosExperiment {
  id: string;
  name: string;
  description: string;
  target_host_ids_json: string;
  target_type: string;
  fault_type: string;
  duration_seconds: number;
  params_json: string;
  status: string;
  started_at: string | null;
  finished_at: string | null;
  created_by: string;
  created_at: string;
}

export interface CreateChaosExperimentInput {
  name: string;
  description?: string;
  target_host_ids_json?: string;
  target_type?: string;
  fault_type: string;
  duration_seconds?: number;
  params_json?: string;
}

export interface ChaosStats {
  total_experiments: number;
  completed: number;
  failed: number;
  running: number;
}
