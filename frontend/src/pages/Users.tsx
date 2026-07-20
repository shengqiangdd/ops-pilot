import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { UserInfo, CreateUserInput } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

const EMPTY_FORM: CreateUserInput = {
  username: '',
  email: '',
  password: '',
  role: 'operator',
};

export function UsersPage() {
  const { token, isAdmin } = useAuthStore();
  const { t } = useI18n();

  const [users, setUsers] = useState<UserInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<CreateUserInput>(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [deleting, setDeleting] = useState<string | null>(null);
  const [editingRole, setEditingRole] = useState<string | null>(null);

  const admin = isAdmin();

  const load = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    setError(null);
    try {
      const list = await api.listUsers(token);
      setUsers(list);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load users');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { load(); }, [load]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      await api.createUser(token!, form);
      setForm(EMPTY_FORM);
      setShowForm(false);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create user');
    } finally {
      setSubmitting(false);
    }
  };

  const handleRoleChange = async (userId: string, newRole: string) => {
    if (!token) return;
    setEditingRole(userId);
    try {
      await api.updateUserRole(token, userId, newRole);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to update role');
    } finally {
      setEditingRole(null);
    }
  };

  const handleDelete = async (userId: string) => {
    if (!window.confirm(t('users.delete_confirm'))) return;
    setDeleting(userId);
    setError(null);
    try {
      await api.deleteUser(token!, userId);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete user');
    } finally {
      setDeleting(null);
    }
  };

  const roleColor = (role: string) => {
    switch (role) {
      case 'admin': return 'bg-purple-500/10 text-purple-600 dark:text-purple-400';
      case 'operator': return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
      case 'viewer': return 'bg-green-500/10 text-green-600 dark:text-green-400';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.users')}
        </h2>
        <div className="flex gap-2">
          <button
            onClick={load}
            disabled={loading}
            className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors"
          >
            {loading ? t('users.loading') : t('users.reload')}
          </button>
          {admin && (
            <button
              onClick={() => setShowForm(!showForm)}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all"
            >
              {showForm ? t('users.cancel') : t('users.add')}
            </button>
          )}
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {!admin && (
        <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-300 dark:border-amber-700 rounded-md-sm px-4 py-3 text-body-medium text-amber-800 dark:text-amber-200">
          {t('users.admin_required')}
        </div>
      )}

      {showForm && admin && (
        <form onSubmit={handleSubmit} className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 space-y-3 animate-slide-up">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('users.username')}</label>
              <input type="text" required value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
                placeholder="admin" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('users.email')}</label>
              <input type="email" required value={form.email} onChange={(e) => setForm({ ...form, email: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
                placeholder="admin@example.com" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('users.password')}</label>
              <input type="password" required value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
                placeholder="min 8 characters" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('users.role')}</label>
              <select value={form.role} onChange={(e) => setForm({ ...form, role: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface">
                <option value="operator">{t('users.role_operator')}</option>
                <option value="admin">{t('users.role_admin')}</option>
                <option value="viewer">{t('users.role_viewer')}</option>
              </select>
            </div>
          </div>
          <div className="flex justify-end">
            <button type="submit" disabled={submitting}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
              {submitting ? t('users.creating') : t('users.create_btn')}
            </button>
          </div>
        </form>
      )}

      <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
        <div className="overflow-x-auto">
          <table className="min-w-full">
            <thead>
              <tr className="border-b border-md-outline-variant">
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('users.col.username')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('users.col.email')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('users.col.role')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('users.col.created')}</th>
                {admin && (
                  <th className="px-4 py-3 text-right text-label-medium text-md-on-surface-variant">{t('users.col.actions')}</th>
                )}
              </tr>
            </thead>
            <tbody>
              {users.map((u) => (
                <tr key={u.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{u.username}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant">{u.email}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium">
                    {admin ? (
                      <select
                        value={u.role}
                        onChange={(e) => handleRoleChange(u.id, e.target.value)}
                        disabled={editingRole === u.id}
                        className={cn(
                          'text-xs font-medium px-2 py-1 rounded-md-sm border border-md-outline bg-md-surface-container-highest',
                          'focus:border-md-primary outline-none',
                          editingRole === u.id && 'opacity-50',
                        )}
                      >
                        <option value="admin">{t('users.role_admin')}</option>
                        <option value="operator">{t('users.role_operator')}</option>
                        <option value="viewer">{t('users.role_viewer')}</option>
                      </select>
                    ) : (
                      <span className={cn('inline-block text-xs font-medium px-2 py-1 rounded-md-sm', roleColor(u.role))}>
                        {t('users.role_' + u.role)}
                      </span>
                    )}
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">
                    {new Date(u.created_at).toLocaleDateString()}
                  </td>
                  {admin && (
                    <td className="whitespace-nowrap px-4 py-3 text-right">
                      <button onClick={() => handleDelete(u.id)} disabled={deleting === u.id}
                        className="text-md-error rounded-md-sm px-2.5 py-1 text-label-large hover:bg-md-error-container/30 disabled:opacity-50 transition-colors">
                        {deleting === u.id ? t('users.deleting') : t('users.delete')}
                      </button>
                    </td>
                  )}
                </tr>
              ))}
              {!loading && users.length === 0 && (
                <tr><td colSpan={admin ? 5 : 4} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('users.empty')}</td></tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
