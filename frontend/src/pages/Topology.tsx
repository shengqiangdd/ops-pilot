import { useCallback, useEffect, useState } from 'react';
import ReactFlow, { Node, Edge, Background, Controls } from 'reactflow';
import 'reactflow/dist/style.css';
import { api } from '../api/client';
import type { TopoGraph } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';

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

  useEffect(() => { load(); }, [load]);

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
      borderRadius: '12px',
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
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">Topology</h2>
        <div className="flex gap-2">
          <button onClick={load} disabled={loading}
            className="border border-md-outline text-md-primary rounded-md-lg px-6 py-2.5 font-medium hover:bg-md-surface-container-high disabled:opacity-50">
            {loading ? 'Loading...' : 'Refresh'}
          </button>
          <button onClick={handleDiscover} disabled={loading}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
            {loading ? 'Discovering...' : 'Discover Topology'}
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      <div className="bg-md-surface-container-low rounded-md-lg shadow-md-1" style={{ height: '500px' }}>
        {nodes.length > 0 ? (
          <ReactFlow nodes={nodes} edges={edges} fitView>
            <Background />
            <Controls />
          </ReactFlow>
        ) : (
          <div className="flex h-full items-center justify-center text-body-medium text-md-on-surface-variant">
            {loading ? 'Loading...' : 'No topology data — click "Discover Topology" to scan'}
          </div>
        )}
      </div>
    </div>
  );
}
