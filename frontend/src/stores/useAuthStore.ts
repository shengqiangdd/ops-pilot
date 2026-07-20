import { create } from 'zustand';

const AUTH_TOKEN_KEY = 'opspilot-auth-token';
const AUTH_USERNAME_KEY = 'opspilot-auth-username';

function loadFromStorage(): { token: string | null; username: string | null } {
  try {
    const token = localStorage.getItem(AUTH_TOKEN_KEY);
    const username = localStorage.getItem(AUTH_USERNAME_KEY);
    return { token, username };
  } catch {
    return { token: null, username: null };
  }
}

function saveToStorage(token: string | null, username: string | null) {
  try {
    if (token) {
      localStorage.setItem(AUTH_TOKEN_KEY, token);
      localStorage.setItem(AUTH_USERNAME_KEY, username ?? '');
    } else {
      localStorage.removeItem(AUTH_TOKEN_KEY);
      localStorage.removeItem(AUTH_USERNAME_KEY);
    }
  } catch { /* ignore */ }
}

interface AuthState {
  token: string | null;
  username: string | null;
  setAuth: (token: string, username: string) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  ...loadFromStorage(),
  setAuth: (token, username) => {
    saveToStorage(token, username);
    set({ token, username });
  },
  logout: () => {
    saveToStorage(null, null);
    set({ token: null, username: null });
  },
}));
