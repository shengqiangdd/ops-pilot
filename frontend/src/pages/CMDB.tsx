import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { CMDBService, CreateServiceInput, ServiceDetail, ConfigVersion } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

type View = 'list' | 'detail' | 'configs';

const EMPTY_FORM: CreateServiceInput = {
  name: '',
  version: '',
  description: '',
  owner: '',
  status: 'active',
};

export function CMDBPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [view, setView] = useState<View>('list');
  const [services, setServices] = useState<CMDBService[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<CreateServiceInput>(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [editing, setEditing] = useState<string | null>(null);
  const [selectedService, setSelectedService] = useState<ServiceDetail | null>(null);
  const [configs, setConfigs] = useState<ConfigVersion[]>([]);

  const loadServices = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    setError(null);
    try {
      const params: Record<string, string> = {};
      if (search) params.search = search;
      const data = await api.listCMDBServices(token, params);
      setServices(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load services');
    } finally {
      setLoading(false);
    }
  }, [token, search]);

  useEffect(() => { loadServices(); }, [loadServices]);

  const loadServiceDetail = async (serviceId: string) => {
    if (!token) return;
    try {
      const detail = await api.getCMDBServiceDetail(token, serviceId);
      setSelectedService(detail);
      setView('detail');
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load service');
    }
  };

  const loadConfigs = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.listConfigVersions(token);
      setConfigs(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load configs');
    }
  }, [token]);

  useEffect(() => {
    if (view === 'configs') loadConfigs();
  }, [view, loadConfigs]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      if (editing) {
        await api.updateCMDBService(token!, editing, form);
      } else {
        await api.createCMDBService(token!, form);
      }
      setForm(EMPTY_FORM);
      setShowForm(false);
      setEditing(null);
      await loadServices();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to save service');
    } finally {
      setSubmitting(false);
    }
  };

  const handleEdit = (service: CMDBService) => {
    setForm({
      name: service.name,
      version: service.version,
      description: service.description,
      owner: service.owner,
      status: service.status,
    });
    setEditing(service.id);
    setShowForm(true);
  };

  const handleDelete = async (id: string) => {
    if (!window.confirm(t('cmdb.delete_confirm'))) return;
    try {
      await api.deleteCMDBService(token!, id);
      await loadServices();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete service');
    }
  };

  const statusColor = (status: string) => {
    switch (status) {
      case 'active': return 'bg-green-500/10 text-green-600 dark:text-green-400';
      case 'deprecated': return 'bg-amber-500/10 text-amber-600 dark:text-amber-400';
      case 'archived': return 'bg-md-surface-container-high text-md-on-surface-variant';
      default: return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
    }
  };

  const roleColor = (role: string) => {
    switch (role) {
      case 'web': return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
      case 'app': return 'bg-green-500/10 text-green-600 dark:text-green-400';
      case 'db': return 'bg-purple-500/10 text-purple-600 dark:text-purple-400';
      case 'cache': return 'bg-amber-500/10 text-amber-600 dark:text-amber-400';
      case 'queue': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };

  return (
    <div className="space-y-4 animate-slide-up">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
            {t('title.cmdb')}
          </h2>
          {view !== 'list' && (
            <button
              onClick={() => { setView('list'); setSelectedService(null); }}
              className="text-sm text-md-primary hover:underline flex items-center gap-1"
            >
              ← {t('cmdb.back_to_list')}
            </button>
          )}
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setView(view === 'configs' ? 'list' : 'configs')}
            className={cn(
              'rounded-md-lg px-4 py-2 text-sm font-medium transition-colors',
              view === 'configs' ? 'bg-md-primary text-md-on-primary' : 'border border-md-outline text-md-primary hover:bg-md-surface-container-high',
            )}
          >
            {t('cmdb.configs')}
          </button>
          {view === 'list' && (
            <button
              onClick={() => { setShowForm(!showForm); setEditing(null); setForm(EMPTY_FORM); }}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all"
            >
              {showForm ? t('cmdb.cancel') : t('cmdb.add_service')}
            </button>
          )}
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {/* Create/Edit Form */}
      {showForm && (
        <form onSubmit={handleSubmit} className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 space-y-3 animate-slide-up">
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('cmdb.name')}</label>
              <input type="text" required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface"
                placeholder="web-api" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('cmdb.version')}</label>
              <input type="text" value={form.version} onChange={(e) => setForm({ ...form, version: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface"
                placeholder="1.0.0" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('cmdb.owner')}</label>
              <input type="text" value={form.owner} onChange={(e) => setForm({ ...form, owner: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface"
                placeholder="team-backend" />
            </div>
            <div className="sm:col-span-2 lg:col-span-1">
              <label className="block text-label-large text-md-on-surface mb-1">{t('cmdb.description')}</label>
              <input type="text" value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface"
                placeholder="Web API service" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('cmdb.status')}</label>
              <select value={form.status} onChange={(e) => setForm({ ...form, status: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface">
                <option value="active">{t('cmdb.status_active')}</option>
                <option value="deprecated">{t('cmdb.status_deprecated')}</option>
                <option value="archived">{t('cmdb.status_archived')}</option>
              </select>
            </div>
          </div>
          <div className="flex justify-end gap-2">
            <button type="button" onClick={() => { setShowForm(false); setEditing(null); }}
              className="px-4 py-2 text-sm font-medium rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">
              {t('cmdb.cancel')}
            </button>
            <button type="submit" disabled={submitting}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
              {submitting ? t('cmdb.saving') : t('cmdb.save')}
            </button>
          </div>
        </form>
      )}

      {/* List View */}
      {view === 'list' && (
        <>
          {/* Search */}
          <div className="glass-card rounded-md-xl p-4">
            <div className="flex gap-3">
              <input
                type="text"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder={t('cmdb.search_placeholder')}
                className="flex-1 bg-md-surface-container-highest rounded-md-sm px-4 py-2 text-sm border border-md-outline focus:border-md-primary outline-none text-md-on-surface"
              />
            </div>
          </div>

          {/* Services Table */}
          <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
            <div className="overflow-x-auto">
              <table className="min-w-full">
                <thead>
                  <tr className="border-b border-md-outline-variant">
                    <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cmdb.col.name')}</th>
                    <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cmdb.col.version')}</th>
                    <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cmdb.col.owner')}</th>
                    <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cmdb.col.status')}</th>
                    <th className="px-4 py-3 text-right text-label-medium text-md-on-surface-variant">{t('cmdb.col.actions')}</th>
                  </tr>
                </thead>
                <tbody>
                  {services.map((svc) => (
                    <tr key={svc.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors cursor-pointer"
                        onClick={() => loadServiceDetail(svc.id)}>
                      <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{svc.name}</td>
                      <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant font-mono">{svc.version || '-'}</td>
                      <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{svc.owner || '-'}</td>
                      <td className="whitespace-nowrap px-4 py-3 text-body-small">
                        <span className={cn('inline-block text-xs font-medium px-2 py-1 rounded-md-sm', statusColor(svc.status))}>
                          {svc.status}
                        </span>
                      </td>
                      <td className="whitespace-nowrap px-4 py-3 text-right" onClick={(e) => e.stopPropagation()}>
                        <div className="flex items-center justify-end gap-2">
                          <button onClick={() => handleEdit(svc)}
                            className="text-md-primary text-label-large hover:bg-md-primary-container/30 px-2 py-1 rounded-md-sm transition-colors">
                            {t('cmdb.edit')}
                          </button>
                          <button onClick={() => handleDelete(svc.id)}
                            className="text-md-error text-label-large hover:bg-md-error-container/30 px-2 py-1 rounded-md-sm transition-colors">
                            {t('cmdb.delete')}
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))}
                  {!loading && services.length === 0 && (
                    <tr><td colSpan={5} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('cmdb.empty')}</td></tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </>
      )}

      {/* Detail View */}
      {view === 'detail' && selectedService && (
        <div className="space-y-4">
          {/* Service Info */}
          <div className="glass-card rounded-md-xl p-5">
            <div className="flex items-start justify-between mb-4">
              <div>
                <h3 className="text-title-large font-semibold text-md-on-surface">{selectedService.service.name}</h3>
                <p className="text-body-medium text-md-on-surface-variant mt-1">{selectedService.service.description || t('cmdb.no_description')}</p>
              </div>
              <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', statusColor(selectedService.service.status))}>
                {selectedService.service.status}
              </span>
            </div>
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
              <div>
                <p className="text-label-small text-md-on-surface-variant">{t('cmdb.version')}</p>
                <p className="text-body-medium font-medium text-md-on-surface">{selectedService.service.version || '-'}</p>
              </div>
              <div>
                <p className="text-label-small text-md-on-surface-variant">{t('cmdb.owner')}</p>
                <p className="text-body-medium font-medium text-md-on-surface">{selectedService.service.owner || '-'}</p>
              </div>
              <div>
                <p className="text-label-small text-md-on-surface-variant">{t('cmdb.col.created')}</p>
                <p className="text-body-medium text-md-on-surface">{new Date(selectedService.service.created_at).toLocaleDateString()}</p>
              </div>
              <div>
                <p className="text-label-small text-md-on-surface-variant">{t('cmdb.col.updated')}</p>
                <p className="text-body-medium text-md-on-surface">{new Date(selectedService.service.updated_at).toLocaleDateString()}</p>
              </div>
            </div>
          </div>

          {/* Hosts */}
          <div className="glass-card rounded-md-xl p-5">
            <h4 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('cmdb.hosts')}</h4>
            {selectedService.hosts.length > 0 ? (
              <div className="space-y-2">
                {selectedService.hosts.map((sh) => (
                  <div key={sh.id} className="flex items-center gap-3 px-3 py-2 rounded-md-lg glass-card">
                    <span className="text-lg">🖥️</span>
                    <span className="flex-1 text-body-medium text-md-on-surface font-mono">{sh.host_id}</span>
                    <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', roleColor(sh.role))}>
                      {sh.role}
                    </span>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-body-small text-md-on-surface-variant">{t('cmdb.no_hosts')}</p>
            )}
          </div>

          {/* Dependencies */}
          <div className="glass-card rounded-md-xl p-5">
            <h4 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('cmdb.dependencies')}</h4>
            {selectedService.dependencies.length > 0 ? (
              <div className="space-y-2">
                {selectedService.dependencies.map((dep) => {
                  const isSource = dep.source_service_id === selectedService.service.id;
                  const otherId = isSource ? dep.target_service_id : dep.source_service_id;
                  return (
                    <div key={dep.id} className="flex items-center gap-3 px-3 py-2 rounded-md-lg glass-card">
                      <span className="text-lg">{isSource ? '→' : '←'}</span>
                      <span className="flex-1 text-body-medium text-md-on-surface font-mono">{otherId}</span>
                      <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm',
                        dep.dependency_type === 'hard' ? 'bg-red-500/10 text-red-600' : 'bg-blue-500/10 text-blue-600'
                      )}>
                        {dep.dependency_type}
                      </span>
                    </div>
                  );
                })}
              </div>
            ) : (
              <p className="text-body-small text-md-on-surface-variant">{t('cmdb.no_dependencies')}</p>
            )}
          </div>
        </div>
      )}

      {/* Configs View */}
      {view === 'configs' && (
        <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="border-b border-md-outline-variant">
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cmdb.config_col.service')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cmdb.config_col.version')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cmdb.config_col.changed_by')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cmdb.config_col.note')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cmdb.config_col.time')}</th>
                </tr>
              </thead>
              <tbody>
                {configs.map((cfg) => (
                  <tr key={cfg.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                    <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface font-mono">{cfg.service_id}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">v{cfg.version}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{cfg.changed_by || '-'}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{cfg.change_note || '-'}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">
                      {new Date(cfg.created_at).toLocaleString()}
                    </td>
                  </tr>
                ))}
                {!loading && configs.length === 0 && (
                  <tr><td colSpan={5} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('cmdb.no_configs')}</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
