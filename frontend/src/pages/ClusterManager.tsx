import { useState, useEffect } from 'react';

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
      if (!resp.ok) throw new Error('创建失败');
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
    if (!confirm(`确定删除集群 "${name}"？`)) return;
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
          <h2 className="text-lg font-semibold text-md-on-surface">Multi-Cluster 管理</h2>
          <button
            onClick={() => setShowForm(!showForm)}
            className="px-4 py-1.5 rounded-md-lg text-sm font-medium bg-md-primary text-md-on-primary hover:opacity-90 transition-all"
          >
            {showForm ? '取消' : '+ 注册集群'}
          </button>
        </div>

        {error && (
          <div className="mb-4 p-3 rounded-md-lg bg-red-50 text-red-600 text-sm">{error}</div>
        )}

        {/* 注册表单 */}
        {showForm && (
          <div className="mb-6 p-4 rounded-md-lg border border-md-outline-variant bg-md-surface-container/30 space-y-3">
            <input
              type="text"
              value={newName}
              onChange={e => setNewName(e.target.value)}
              placeholder="集群名称"
              className="w-full px-3 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
            />
            <input
              type="text"
              value={newApiServer}
              onChange={e => setNewApiServer(e.target.value)}
              placeholder="API Server 地址 (https://...)"
              className="w-full px-3 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
            />
            <input
              type="password"
              value={newToken}
              onChange={e => setNewToken(e.target.value)}
              placeholder="认证 Token（可选）"
              className="w-full px-3 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
            />
            <button
              onClick={handleCreate}
              className="px-4 py-1.5 rounded-md-lg text-sm font-medium bg-md-primary text-md-on-primary hover:opacity-90"
            >
              注册
            </button>
          </div>
        )}

        {/* 集群列表 */}
        {loading ? (
          <div className="text-center py-8 text-md-on-surface-variant">加载中...</div>
        ) : clusters.length === 0 ? (
          <div className="text-center py-8 text-md-on-surface-variant">暂无集群，点击"注册集群"添加</div>
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
                      检查状态
                    </button>
                    <button
                      onClick={() => handleDelete(cluster.id, cluster.name)}
                      className="px-3 py-1 rounded-md text-xs font-medium bg-red-50 text-red-600 hover:bg-red-100"
                    >
                      删除
                    </button>
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-2 text-xs">
                  <div>
                    <span className="text-md-on-surface-variant">API Server:</span>
                    <span className="ml-2 text-md-on-surface font-mono">{cluster.api_server}</span>
                  </div>
                  <div>
                    <span className="text-md-on-surface-variant">创建时间:</span>
                    <span className="ml-2 text-md-on-surface">{cluster.created_at}</span>
                  </div>
                </div>

                {/* 状态详情 */}
                {statusMap[cluster.id] && (
                  <div className="mt-3 p-3 rounded-md-lg bg-md-surface/50 border border-md-outline-variant/20">
                    <div className="grid grid-cols-4 gap-2 text-xs">
                      <div>
                        <span className="text-md-on-surface-variant">API 可达:</span>
                        <span className={`ml-1 font-medium ${statusMap[cluster.id].api_reachable ? 'text-green-600' : 'text-red-600'}`}>
                          {statusMap[cluster.id].api_reachable ? '是' : '否'}
                        </span>
                      </div>
                      <div>
                        <span className="text-md-on-surface-variant">节点数:</span>
                        <span className="ml-1 text-md-on-surface font-medium">{statusMap[cluster.id].node_count}</span>
                      </div>
                      <div>
                        <span className="text-md-on-surface-variant">版本:</span>
                        <span className="ml-1 text-md-on-surface font-mono">{statusMap[cluster.id].version}</span>
                      </div>
                      <div>
                        <span className="text-md-on-surface-variant">状态:</span>
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
