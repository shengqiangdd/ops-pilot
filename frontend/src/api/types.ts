export interface ModuleInfo {
  name: string;
  version: string;
  description: string;
  enabled: boolean;
}

export type HealthStatus =
  | { Healthy: null }
  | { Degraded: { reason: string } }
  | { Unhealthy: { reason: string } };

export interface ModuleHealth {
  name: string;
  status: HealthStatus;
  enabled: boolean;
}

export interface ModuleConfig {
  [key: string]: unknown;
}

// ── Host types ──────────────────────────────────────────────────────────

export type HostStatus = 'online' | 'offline' | 'unknown' | 'maintenance';

export interface Host {
  id: string;
  name: string;
  address: string;
  port: number;
  username: string;
  auth_method: string;
  status: HostStatus;
  created_at: string;
  updated_at: string;
}

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
