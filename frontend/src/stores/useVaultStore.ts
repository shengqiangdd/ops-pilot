import { create } from 'zustand';
import { api } from '../api/client';
import { useAuthStore } from './useAuthStore';

interface VaultState {
  isUnlocked: boolean;
  hasPassphrase: boolean;
  checkingStatus: boolean;
  error: string | null;

  checkStatus: () => Promise<void>;
  unlock: (loginPassword: string, passphrase: string) => Promise<void>;
  lock: () => Promise<void>;
  setPassphrase: (loginPassword: string, passphrase: string, confirm: string) => Promise<void>;
}

export const useVaultStore = create<VaultState>((set) => ({
  isUnlocked: false,
  hasPassphrase: false,
  checkingStatus: false,
  error: null,

  checkStatus: async () => {
    const token = useAuthStore.getState().token;
    if (!token) return;
    set({ checkingStatus: true });
    try {
      const status = await api.getVaultStatus(token);
      set({ isUnlocked: status.unlocked, hasPassphrase: status.has_passphrase, error: null });
    } catch {
      // Silent fail — vault status check is best-effort
    } finally {
      set({ checkingStatus: false });
    }
  },

  unlock: async (loginPassword, passphrase) => {
    const token = useAuthStore.getState().token;
    if (!token) throw new Error('Not authenticated');
    set({ error: null });
    try {
      await api.unlockVault(token, loginPassword, passphrase);
      set({ isUnlocked: true });
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Unlock failed';
      set({ error: msg });
      throw e;
    }
  },

  lock: async () => {
    const token = useAuthStore.getState().token;
    if (!token) return;
    await api.lockVault(token);
    set({ isUnlocked: false });
  },

  setPassphrase: async (loginPassword, passphrase, confirm) => {
    const token = useAuthStore.getState().token;
    if (!token) throw new Error('Not authenticated');
    set({ error: null });
    try {
      await api.setVaultPassphrase(token, loginPassword, passphrase, confirm);
      set({ hasPassphrase: true });
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Failed to set passphrase';
      set({ error: msg });
      throw e;
    }
  },
}));
