import { useEffect, useState } from 'react';
import { useVaultStore } from '../stores/useVaultStore';
import { cn } from '../lib/cn';

export function VaultPage() {
  const { isUnlocked, hasPassphrase, error, checkStatus, unlock, lock, setPassphrase } =
    useVaultStore();

  const [loginPassword, setLoginPassword] = useState('');
  const [passphraseValue, setPassphraseValue] = useState('');
  const [confirm, setConfirm] = useState('');
  const [loading, setLoading] = useState(false);
  const [localError, setLocalError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => {
    checkStatus();
  }, [checkStatus]);

  const handleSetPassphrase = async (e: React.FormEvent) => {
    e.preventDefault();
    setLocalError(null);
    setSuccess(null);
    if (passphraseValue !== confirm) {
      setLocalError('Passphrases do not match');
      return;
    }
    setLoading(true);
    try {
      await setPassphrase(loginPassword, passphraseValue, confirm);
      setSuccess('Vault passphrase set successfully');
      setLoginPassword('');
      setPassphraseValue('');
      setConfirm('');
    } catch (err) {
      setLocalError(err instanceof Error ? err.message : 'Failed');
    } finally {
      setLoading(false);
    }
  };

  const handleUnlock = async (e: React.FormEvent) => {
    e.preventDefault();
    setLocalError(null);
    setLoading(true);
    try {
      await unlock(loginPassword, passphraseValue);
      setLoginPassword('');
      setPassphraseValue('');
    } catch (err) {
      setLocalError(err instanceof Error ? err.message : 'Failed');
    } finally {
      setLoading(false);
    }
  };

  const handleLock = async () => {
    await lock();
    setSuccess('Vault locked');
  };

  const displayError = localError || error;

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold text-gray-900">Vault</h2>
      <p className="text-sm text-gray-500">
        The vault encrypts your host credentials with a per-user key derived from
        your passphrase. Your passphrase is never stored — only a verification hash.
      </p>

      {displayError && (
        <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{displayError}</div>
      )}
      {success && (
        <div className="rounded-md bg-green-50 p-3 text-sm text-green-700">{success}</div>
      )}

      {/* ── Not set up yet ─────────────────────────────────────────────── */}
      {!hasPassphrase && (
        <form
          onSubmit={handleSetPassphrase}
          className="rounded-lg border border-gray-200 bg-white p-4 space-y-3"
        >
          <h3 className="text-sm font-medium text-gray-900">Set Vault Passphrase</h3>
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Login Password
              </label>
              <input
                type="password"
                required
                value={loginPassword}
                onChange={(e) => setLoginPassword(e.target.value)}
                className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                New Passphrase
              </label>
              <input
                type="password"
                required
                minLength={8}
                value={passphraseValue}
                onChange={(e) => setPassphraseValue(e.target.value)}
                className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Confirm Passphrase
              </label>
              <input
                type="password"
                required
                minLength={8}
                value={confirm}
                onChange={(e) => setConfirm(e.target.value)}
                className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              />
            </div>
          </div>
          <div className="flex justify-end">
            <button
              type="submit"
              disabled={loading}
              className={cn(
                'rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white',
                'hover:bg-blue-700 disabled:opacity-50',
              )}
            >
              {loading ? 'Setting...' : 'Set Passphrase'}
            </button>
          </div>
        </form>
      )}

      {/* ── Set up but locked ──────────────────────────────────────────── */}
      {hasPassphrase && !isUnlocked && (
        <form
          onSubmit={handleUnlock}
          className="rounded-lg border border-gray-200 bg-white p-4 space-y-3"
        >
          <h3 className="text-sm font-medium text-gray-900">Unlock Vault</h3>
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Login Password
              </label>
              <input
                type="password"
                required
                value={loginPassword}
                onChange={(e) => setLoginPassword(e.target.value)}
                className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Vault Passphrase
              </label>
              <input
                type="password"
                required
                value={passphraseValue}
                onChange={(e) => setPassphraseValue(e.target.value)}
                className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              />
            </div>
          </div>
          <div className="flex justify-end">
            <button
              type="submit"
              disabled={loading}
              className={cn(
                'rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white',
                'hover:bg-blue-700 disabled:opacity-50',
              )}
            >
              {loading ? 'Unlocking...' : 'Unlock'}
            </button>
          </div>
        </form>
      )}

      {/* ── Unlocked ───────────────────────────────────────────────────── */}
      {hasPassphrase && isUnlocked && (
        <div className="rounded-lg border border-green-200 bg-green-50 p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-green-800">Vault Unlocked</p>
              <p className="text-xs text-green-600">
                Your host credentials are decrypted and available.
              </p>
            </div>
            <button
              onClick={handleLock}
              className={cn(
                'rounded-md border border-red-300 px-3 py-1.5 text-sm font-medium text-red-700',
                'hover:bg-red-50',
              )}
            >
              Lock Vault
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
