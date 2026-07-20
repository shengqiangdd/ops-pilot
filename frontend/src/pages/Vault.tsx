import { useEffect, useState } from 'react';
import { useVaultStore } from '../stores/useVaultStore';

export function VaultPage() {
  const { isUnlocked, hasPassphrase, error, checkStatus, unlock, lock, setPassphrase } = useVaultStore();

  const [loginPassword, setLoginPassword] = useState('');
  const [passphraseValue, setPassphraseValue] = useState('');
  const [confirm, setConfirm] = useState('');
  const [loading, setLoading] = useState(false);
  const [localError, setLocalError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => { checkStatus(); }, [checkStatus]);

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
    <div className="space-y-4 animate-slide-up">
      <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">Vault</h2>
      <p className="text-body-medium text-md-on-surface-variant">
        The vault encrypts your host credentials with a per-user key derived from
        your passphrase. Your passphrase is never stored — only a verification hash.
      </p>

      {displayError && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{displayError}</div>
      )}
      {success && (
        <div className="bg-md-primary-container text-md-on-primary-container rounded-md-sm px-4 py-3 text-body-medium">{success}</div>
      )}

      {!hasPassphrase && (
        <form onSubmit={handleSetPassphrase} className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 space-y-3">
          <h3 className="text-title-medium font-medium text-md-on-surface">Set Vault Passphrase</h3>
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">Login Password</label>
              <input type="password" required value={loginPassword} onChange={(e) => setLoginPassword(e.target.value)}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">New Passphrase</label>
              <input type="password" required minLength={8} value={passphraseValue} onChange={(e) => setPassphraseValue(e.target.value)}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">Confirm Passphrase</label>
              <input type="password" required minLength={8} value={confirm} onChange={(e) => setConfirm(e.target.value)}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface" />
            </div>
          </div>
          <div className="flex justify-end">
            <button type="submit" disabled={loading}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
              {loading ? 'Setting...' : 'Set Passphrase'}
            </button>
          </div>
        </form>
      )}

      {hasPassphrase && !isUnlocked && (
        <form onSubmit={handleUnlock} className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 space-y-3">
          <h3 className="text-title-medium font-medium text-md-on-surface">Unlock Vault</h3>
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">Login Password</label>
              <input type="password" required value={loginPassword} onChange={(e) => setLoginPassword(e.target.value)}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">Vault Passphrase</label>
              <input type="password" required value={passphraseValue} onChange={(e) => setPassphraseValue(e.target.value)}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface" />
            </div>
          </div>
          <div className="flex justify-end">
            <button type="submit" disabled={loading}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
              {loading ? 'Unlocking...' : 'Unlock'}
            </button>
          </div>
        </form>
      )}

      {hasPassphrase && isUnlocked && (
        <div className="bg-md-primary-container text-md-on-primary-container rounded-md-lg p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-title-medium font-medium">Vault Unlocked</p>
              <p className="text-body-medium text-md-on-primary-container/80">
                Your host credentials are decrypted and available.
              </p>
            </div>
            <button onClick={handleLock}
              className="border border-md-outline text-md-error rounded-md-lg px-6 py-2.5 font-medium hover:bg-md-error-container/20 transition-colors">
              Lock Vault
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
