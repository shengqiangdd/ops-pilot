import { create } from 'zustand';

const AUTH_TOKEN_KEY = 'opspilot-auth-token';
const AUTH_USERNAME_KEY = 'opspilot-auth-username';
const AUTH_ROLE_KEY = 'opspilot-auth-role';

function loadFromStorage(): { token: string | null; username: string | null; role: string | null } {
  try {
    const token = localStorage.getItem(AUTH_TOKEN_KEY);
    const username = localStorage.getItem(AUTH_USERNAME_KEY);
    const role = localStorage.getItem(AUTH_ROLE_KEY);
    return { token, username, role };
  } catch {
    return { token: null, username: null, role: null };
  }
}

function saveToStorage(token: string | null, username: string | null, role: string | null) {
  try {
    if (token) {
      localStorage.setItem(AUTH_TOKEN_KEY, token);
      localStorage.setItem(AUTH_USERNAME_KEY, username ?? '');
      localStorage.setItem(AUTH_ROLE_KEY, role ?? 'operator');
    } else {
      localStorage.removeItem(AUTH_TOKEN_KEY);
      localStorage.removeItem(AUTH_USERNAME_KEY);
      localStorage.removeItem(AUTH_ROLE_KEY);
    }
  } catch { /* ignore */ }
}

type Role = 'admin' | 'operator' | 'viewer';

interface AuthState {
  token: string | null;
  username: string | null;
  role: Role | null;
  isAdmin: () => boolean;
  isViewer: () => boolean;
  canWrite: () => boolean;
  setAuth: (token: string, username: string, role?: string) => void;
  setRole: (role: string) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthState>((set, get) => {
  const stored = loadFromStorage();
  return {
    token: stored.token,
    username: stored.username,
    role: (stored.role as Role) ?? null,
    isAdmin: () => get().role === 'admin',
    isViewer: () => get().role === 'viewer',
    canWrite: () => get().role !== 'viewer',
    setAuth: (token, username, role) => {
      const r = (role ?? 'operator') as Role;
      saveToStorage(token, username, r);
      set({ token, username, role: r });
    },
    setRole: (role) => {
      const r = role as Role;
      const state = get();
      saveToStorage(state.token, state.username, r);
      set({ role: r });
    },
    logout: () => {
      saveToStorage(null, null, null);
      set({ token: null, username: null, role: null });
    },
  };
});
