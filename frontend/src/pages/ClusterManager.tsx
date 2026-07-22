import { useState, useEffect } from 'react';
import { useI18n } from '../i18n';

interface Cluster {
  id: string;
  name: string;
  api_server: string;
  token: string | null;
  status: string;
  metrics_json: string | null;
  created_at: string;
  updated_at: string;
}

interface ClusterStatus {
  id: string;
  name: string;
  status: string;
  api_reachable: boolean;
  node_count: number;
  version: string;
}

export function ClusterManagerPage() {
  const { t } = useI18n();
  const [clusters, setClusters] = useState<Cluster[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [newName, setNewName] = useState('');
  const [newApiServer, setNewApiServer] = useState('');
  const [newToken, setNewToken] = useState('');
  const [statusMap, setStatusMap] = useState<Record<string, ClusterStatus>>({});
  const [error, setError] = useState<string | null>(null);

  const fetchClusters = async () => {
    setLoading(true);
    try {
      const resp = await fetch('/api/clusters');
      const data = await resp.json();
      setClusters(data);
    } catch (e: any) {
      setError(e.message);
    }
    setLoading(false);
  };

  useEffect(() => { fetchClusters(); }, []);

  const handleCreate = async () => {
    if (!newName.trim() || !newApiServer.trim()) return;
    try {
      const resp = await fetch('/api/clusters', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: newName,
          api_server: newApiServer,
          token: newToken || null,
        }),
      });
      if (!resp.ok) throw new Error(t('cluster_manager.create_failed'));
      setShowForm(false);
      setNewName('');
      setNewApiServer('');
      setNewToken('');
      fetchClusters();
    } catch (e: any) {
      setError(e.message);
    }
  };

  const handleDelete = async (id: string, name: string) => {
    if (!confirm(t('cluster_manager.delete_confirm').replace('{name}', name))) return;
    try {
      await fetch(`/api/clusters/${id}`, { method: 'DELETE' });
      fetchClusters();
    } catch (e: any) {
      setError(e.message);
    }
  };

  const handleCheckStatus = async (id: string) => {
    try {
      const resp = await fetch(`/api/clusters/${id}/status`);
      const data = await resp.json();
      setStatusMap(prev => ({ ...prev, [id]: data }));
    } catch (e: any) {
      console.error(e);
    }
  };

  const statusColor = (status: string) => {
    switch (status) {
      case 'healthy': case 'online': return 'bg-green-100 text-green-700';
      case 'unhealthy': case 'offline': return 'bg-red-100 text-red-700';
      default: return 'bg-gray-100 text-gray-600';
    }
  };

  return (
    <div className="space-y-6">
      <div className="glass-card p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-md-on-surface">{t('cluster_manager.title')}</h2>
          <button
            onClick={() => setShowForm(!showForm)}
            className="px-4 py-1.5 rounded-md-lg text-sm font-medium bg-md-primary text-md-on-primary hover:opacity-90 transition-all"
          >
            {showForm ? t('cluster_manager.cancel') : '+ ' + t('cluster_manager.register')}
          </button>
        </div>

        {error && (
          <div className="mb-4 p-3 rounded-md-lg bg-red-50 text-red-600 text-sm">{error}</div>
        )}

        {/* Register form */}
        {showForm && (
          <div className="mb-6 p-4 rounded-md-lg border border-md-outline-variant bg-md-surface-container/30 space-y-3">
            <input
              type="text"
              value={newName}
              onChange={e => setNewName(e.target.value)}
              placeholder={t('cluster_manager.name')}
              className="w-full px-3 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
            />
            <input
              type="text"
              value={newApiServer}
              onChange={e => setNewApiServer(e.target.value)}
              placeholder={t('cluster_manager.api_server')}
              className="w-full px-3 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
            />
            <input
              type="password"
              value={newToken}
              onChange={e => setNewToken(e.target.value)}
              placeholder={t('cluster_manager.token')}
              className="w-full px-3 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
            />
            <button
              onClick={handleCreate}
              className="px-4 py-1.5 rounded-md-lg text-sm font-medium bg-md-primary text-md-on-primary hover:opacity-90"
            >
              {t('cluster_manager.register_btn')}
            </button>
          </div>
        )}

        {/* Cluster list */}
        {loading ? (
          <div className="text-center py-8 text-md-on-surface-variant">{t('cluster_manager.loading')}</div>
        ) : clusters.length === 0 ? (
          <div className="text-center py-8 text-md-on-surface-variant">{t('cluster_manager.add_hint')}</div>
        ) : (
          <div className="space-y-3">
            {clusters.map(cluster => (
              <div key={cluster.id} className="p-4 rounded-md-lg bg-md-surface-container/30 border border-md-outline-variant/30">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-3">
                    <h4 className="text-sm font-medium text-md-on-surface">{cluster.name}</h4>
                    <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${statusColor(cluster.status)}`}>
                      {cluster.status}
                    </span>
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={() => handleCheckStatus(cluster.id)}
                      className="px-3 py-1 rounded-md text-xs font-medium bg-md-surface-container text-md-on-surface hover:glass-card"
                    >
                      {t('cluster_manager.check_status')}
                    </button>
                    <button
                      onClick={() => handleDelete(cluster.id, cluster.name)}
                      className="px-3 py-1 rounded-md text-xs font-medium bg-red-50 text-red-600 hover:bg-red-100"
                    >
                      {t('cluster_manager.delete')}
                    </button>
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-2 text-xs">
                  <div>
                    <span className="text-md-on-surface-variant">{t('cluster_manager.api_server')}:</span>
                    <span className="ml-2 text-md-on-surface font-mono">{cluster.api_server}</span>
                  </div>
                  <div>
                    <span className="text-md-on-surface-variant">{t('cluster_manager.created_at')}:</span>
                    <span className="ml-2 text-md-on-surface">{cluster.created_at}</span>
                  </div>
                </div>

                {/* Status details */}
                {statusMap[cluster.id] && (
                  <div className="mt-3 p-3 rounded-md-lg bg-md-surface/50 border border-md-outline-variant/20">
                    <div className="grid grid-cols-4 gap-2 text-xs">
                      <div>
                        <span className="text-md-on-surface-variant">{t('cluster_manager.api_reachable')}:</span>
                        <span className={`ml-1 font-medium ${statusMap[cluster.id].api_reachable ? 'text-green-600' : 'text-red-600'}`}>
                          {statusMap[cluster.id].api_reachable ? t('cluster_manager.yes') : t('cluster_manager.no')}
                        </span>
                      </div>
                      <div>
                        <span className="text-md-on-surface-variant">{t('cluster_manager.node_count')}:</span>
                        <span className="ml-1 text-md-on-surface font-medium">{statusMap[cluster.id].node_count}</span>
                      </div>
                      <div>
                        <span className="text-md-on-surface-variant">{t('cluster_manager.version')}:</span>
                        <span className="ml-1 text-md-on-surface font-mono">{statusMap[cluster.id].version}</span>
                      </div>
                      <div>
                        <span className="text-md-on-surface-variant">{t('cluster_manager.status')}:</span>
                        <span className={`ml-1 font-medium ${statusColor(statusMap[cluster.id].status)}`}>
                          {statusMap[cluster.id].status}
                        </span>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
