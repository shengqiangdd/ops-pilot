import type { ModuleInfo, ModuleHealth, HealthStatus, ModuleConfig } from './types';

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
  return res.json();
}

export const api = {
  listModules: () => request<ModuleInfo[]>('/modules'),

  getModule: (name: string) => request<ModuleInfo>(`/modules/${encodeURIComponent(name)}`),

  enableModule: (name: string) =>
    request<{ enabled: boolean }>(`/modules/${encodeURIComponent(name)}/enable`, { method: 'POST' }),

  disableModule: (name: string) =>
    request<{ enabled: boolean }>(`/modules/${encodeURIComponent(name)}/disable`, { method: 'POST' }),

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
};
