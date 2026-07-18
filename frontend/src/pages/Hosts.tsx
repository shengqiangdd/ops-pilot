import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { Host, CreateHostInput } from '../api/types';
import { useVaultStore } from '../stores/useVaultStore';
import { cn } from '../lib/cn';

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

  useEffect(() => {
    load();
  }, [load]);

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
      case 'online':
        return 'bg-green-500';
      case 'offline':
        return 'bg-red-500';
      case 'maintenance':
        return 'bg-yellow-500';
      default:
        return 'bg-gray-400';
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-900">Hosts</h2>
        <div className="flex gap-2">
          <button
            onClick={load}
            disabled={loading}
            className={cn(
              'rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-700',
              'hover:bg-gray-50 disabled:opacity-50',
            )}
          >
            {loading ? 'Loading...' : 'Reload'}
          </button>
          <button
            onClick={() => setShowForm(!showForm)}
            className={cn(
              'rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white',
              'hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2',
            )}
          >
            {showForm ? 'Cancel' : 'Add Host'}
          </button>
        </div>
      </div>

      {error && (
        <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>
      )}

      {!isUnlocked && (
        <div className="rounded-md bg-amber-50 border border-amber-200 p-3 text-sm text-amber-800">
          Vault is locked. Host credentials are encrypted at rest. Go to the{' '}
          <strong>Vault</strong> tab to unlock and manage credentials.
        </div>
      )}

      {showForm && (
        <form
          onSubmit={handleSubmit}
          className="rounded-lg border border-gray-200 bg-white p-4 space-y-3"
        >
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Name</label>
              <input
                type="text"
                required
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
                className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                placeholder="web-server-1"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Address</label>
              <input
                type="text"
                required
                value={form.address}
                onChange={(e) => setForm({ ...form, address: e.target.value })}
                className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                placeholder="192.168.1.10"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Port</label>
              <input
                type="number"
                value={form.port ?? ''}
                onChange={(e) =>
                  setForm({ ...form, port: e.target.value ? Number(e.target.value) : undefined })
                }
                className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                placeholder="22"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Username</label>
              <input
                type="text"
                required
                value={form.username}
                onChange={(e) => setForm({ ...form, username: e.target.value })}
                className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                placeholder="root"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Auth Method</label>
              <select
                value={form.auth_method}
                onChange={(e) => setForm({ ...form, auth_method: e.target.value })}
                className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              >
                <option value="key">SSH Key</option>
                <option value="password">Password</option>
              </select>
            </div>
          </div>
          <div className="flex justify-end">
            <button
              type="submit"
              disabled={submitting}
              className={cn(
                'rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white',
                'hover:bg-blue-700 disabled:opacity-50',
              )}
            >
              {submitting ? 'Creating...' : 'Create Host'}
            </button>
          </div>
        </form>
      )}

      <div className="overflow-hidden rounded-lg border border-gray-200">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                Name
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                Address
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                Port
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                Status
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                Auth
              </th>
              <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-gray-500">
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200 bg-white">
            {hosts.map((h) => (
              <tr key={h.id}>
                <td className="whitespace-nowrap px-4 py-3 text-sm font-medium text-gray-900">
                  {h.name}
                </td>
                <td className="whitespace-nowrap px-4 py-3 text-sm text-gray-500">
                  {h.address}
                </td>
                <td className="whitespace-nowrap px-4 py-3 text-sm text-gray-500">{h.port}</td>
                <td className="whitespace-nowrap px-4 py-3 text-sm">
                  <span className="inline-flex items-center gap-1.5">
                    <span className={cn('h-2 w-2 rounded-full', statusColor(h.status))} />
                    {h.status}
                  </span>
                </td>
                <td className="whitespace-nowrap px-4 py-3 text-sm text-gray-500">
                  {h.auth_method}
                </td>
                <td className="whitespace-nowrap px-4 py-3 text-right">
                  <button
                    onClick={() => handleDelete(h.id)}
                    disabled={deleting === h.id}
                    className={cn(
                      'rounded-md px-2.5 py-1 text-xs font-medium text-red-600',
                      'hover:bg-red-50 disabled:opacity-50',
                    )}
                  >
                    {deleting === h.id ? 'Deleting...' : 'Delete'}
                  </button>
                </td>
              </tr>
            ))}
            {!loading && hosts.length === 0 && (
              <tr>
                <td colSpan={6} className="px-4 py-8 text-center text-sm text-gray-500">
                  No hosts configured
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
