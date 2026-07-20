import { useCallback, useEffect, useState } from 'react';
import ReactFlow, { Node, Edge, Background, Controls } from 'reactflow';
import 'reactflow/dist/style.css';
import { api } from '../api/client';
import type { TopoGraph } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

const NODE_COLORS: Record<string, string> = {
  Host: '#3b82f6',
  Service: '#10b981',
  Container: '#8b5cf6',
  LoadBalancer: '#f59e0b',
  Database: '#ef4444',
  External: '#6b7280',
};

export function TopologyPage() {
  const { token } = useAuthStore();
  const [graph, setGraph] = useState<TopoGraph | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    setError(null);
    try {
      const data = await api.getTopoGraph(token);
      setGraph(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load topology');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => {
    load();
  }, [load]);

  const handleDiscover = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      await api.discoverTopo(token);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Discovery failed');
    } finally {
      setLoading(false);
    }
  }, [token, load]);

  const nodes: Node[] = (graph?.nodes || []).map((n) => ({
    id: n.id,
    position: { x: Math.random() * 600, y: Math.random() * 400 },
    data: { label: `${n.label}\n(${n.kind})` },
    style: {
      background: NODE_COLORS[n.kind] || '#6b7280',
      color: '#fff',
      borderRadius: '8px',
      padding: '10px 16px',
      fontSize: '12px',
      fontWeight: 600,
    },
  }));

  const edges: Edge[] = (graph?.edges || []).map((e, i) => ({
    id: `e${i}`,
    source: e.source,
    target: e.target,
    label: e.label,
    animated: true,
    style: { stroke: '#94a3b8' },
  }));

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-900">网络拓扑</h2>
        <div className="flex gap-2">
          <button onClick={load} disabled={loading} className={cn('rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50')}>
            {loading ? '加载中...' : '刷新'}
          </button>
          <button onClick={handleDiscover} disabled={loading} className={cn('rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50')}>
            {loading ? '探测中...' : '发现拓扑'}
          </button>
        </div>
      </div>

      {error && <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>}

      <div className="rounded-lg border border-gray-200 bg-white shadow-sm" style={{ height: '500px' }}>
        {nodes.length > 0 ? (
          <ReactFlow nodes={nodes} edges={edges} fitView>
            <Background />
            <Controls />
          </ReactFlow>
        ) : (
          <div className="flex h-full items-center justify-center text-sm text-gray-500">
            {loading ? '加载中...' : '暂无拓扑数据，点击"发现拓扑"开始扫描'}
          </div>
        )}
      </div>
    </div>
  );
}
