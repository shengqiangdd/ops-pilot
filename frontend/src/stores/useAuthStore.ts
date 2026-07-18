import { create } from 'zustand';

interface AuthState {
  token: string | null;
  username: string | null;
  setAuth: (token: string, username: string) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  token: null,
  username: null,
  setAuth: (token, username) => set({ token, username }),
  logout: () => set({ token: null, username: null }),
}));
