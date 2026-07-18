import type {
  ModuleInfo,
  ModuleHealth,
  HealthStatus,
  ModuleConfig,
  Host,
  CreateHostInput,
  AgentSession,
  AgentResponse,
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
};
