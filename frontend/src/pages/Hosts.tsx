import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { Host, CreateHostInput } from '../api/types';
import { useVaultStore } from '../stores/useVaultStore';

const EMPTY_FORM: CreateHostInput = {
  name: '',
  address: '',
  port: 22,
  username: 'root',
  auth_method: 'key',
};

export function HostsPage() {
  const [hosts, setHosts] = useState<Host[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<CreateHostInput>(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [deleting, setDeleting] = useState<string | null>(null);
  const { isUnlocked } = useVaultStore();

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await api.listHosts();
      setHosts(list);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load hosts');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      await api.createHost(form);
      setForm(EMPTY_FORM);
      setShowForm(false);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create host');
    } finally {
      setSubmitting(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!window.confirm('Are you sure you want to delete this host?')) return;
    setDeleting(id);
    setError(null);
    try {
      await api.deleteHost(id);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete host');
    } finally {
      setDeleting(null);
    }
  };

  const statusColor = (status: Host['status']) => {
    switch (status) {
      case 'online': return 'bg-green-500';
      case 'offline': return 'bg-md-error';
      case 'maintenance': return 'bg-amber-500';
      default: return 'bg-md-outline';
    }
  };

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">Hosts</h2>
        <div className="flex gap-2">
          <button
            onClick={load}
            disabled={loading}
            className="border border-md-outline text-md-primary rounded-md-lg px-6 py-2.5 font-medium hover:bg-md-surface-container-high disabled:opacity-50"
          >
            {loading ? 'Loading...' : 'Reload'}
          </button>
          <button
            onClick={() => setShowForm(!showForm)}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all"
          >
            {showForm ? 'Cancel' : 'Add Host'}
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {!isUnlocked && (
        <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-300 dark:border-amber-700 rounded-md-sm px-4 py-3 text-body-medium text-amber-800 dark:text-amber-200">
          Vault is locked. Host credentials are encrypted at rest. Go to the{' '}
          <strong>Vault</strong> tab to unlock and manage credentials.
        </div>
      )}

      {showForm && (
        <form onSubmit={handleSubmit} className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 space-y-3 animate-slide-up">
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">Name</label>
              <input type="text" required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
                placeholder="web-server-1" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">Address</label>
              <input type="text" required value={form.address} onChange={(e) => setForm({ ...form, address: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
                placeholder="192.168.1.10" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">Port</label>
              <input type="number" value={form.port ?? ''} onChange={(e) => setForm({ ...form, port: e.target.value ? Number(e.target.value) : undefined })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
                placeholder="22" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">Username</label>
              <input type="text" required value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
                placeholder="root" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">Auth Method</label>
              <select value={form.auth_method} onChange={(e) => setForm({ ...form, auth_method: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface">
                <option value="key">SSH Key</option>
                <option value="password">Password</option>
              </select>
            </div>
          </div>
          <div className="flex justify-end">
            <button type="submit" disabled={submitting}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
              {submitting ? 'Creating...' : 'Create Host'}
            </button>
          </div>
        </form>
      )}

      <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
        <table className="min-w-full">
          <thead>
            <tr className="border-b border-md-outline-variant">
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Name</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Address</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Port</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Status</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Auth</th>
              <th className="px-4 py-3 text-right text-label-medium text-md-on-surface-variant">Actions</th>
            </tr>
          </thead>
          <tbody>
            {hosts.map((h) => (
              <tr key={h.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{h.name}</td>
                <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant">{h.address}</td>
                <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant">{h.port}</td>
                <td className="whitespace-nowrap px-4 py-3 text-body-medium">
                  <span className="inline-flex items-center gap-1.5">
                    <span className={`h-2 w-2 rounded-full ${statusColor(h.status)}`} />
                    {h.status}
                  </span>
                </td>
                <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant">{h.auth_method}</td>
                <td className="whitespace-nowrap px-4 py-3 text-right">
                  <button onClick={() => handleDelete(h.id)} disabled={deleting === h.id}
                    className="text-md-error rounded-md-sm px-2.5 py-1 text-label-large hover:bg-md-error-container/30 disabled:opacity-50 transition-colors">
                    {deleting === h.id ? 'Deleting...' : 'Delete'}
                  </button>
                </td>
              </tr>
            ))}
            {!loading && hosts.length === 0 && (
              <tr><td colSpan={6} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">No hosts configured</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
