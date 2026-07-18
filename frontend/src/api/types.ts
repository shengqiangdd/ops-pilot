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
