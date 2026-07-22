import { useState, useEffect } from 'react';
import { useI18n } from '../i18n';

interface TraceSummary {
  trace_id: string;
  span_count: number;
  root_operation: string;
  root_service: string;
  start_time: number;
  duration_ms: number;
  status: string;
}

interface TraceSpan {
  trace_id: string;
  span_id: string;
  parent_span_id: string | null;
  operation_name: string;
  service: string;
  start_time: number;
  end_time: number;
  duration_ms: number;
  status: string;
  tags_json: string | null;
}

interface TraceTreeNode {
  span: TraceSpan;
  children: TraceTreeNode[];
}

export function TraceExplorerPage() {
  const { t } = useI18n();
  const [services, setServices] = useState<string[]>([]);
  const [selectedService, setSelectedService] = useState('');
  const [traces, setTraces] = useState<TraceSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedTrace, setSelectedTrace] = useState<string | null>(null);
  const [traceTree, setTraceTree] = useState<TraceTreeNode | null>(null);
  const [treeLoading, setTreeLoading] = useState(false);

  const fetchServices = async () => {
    try {
      const resp = await fetch('/api/otel/services');
      const data = await resp.json();
      setServices(data);
    } catch (e: any) {
      console.error(e);
    }
  };

  const fetchTraces = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams();
      if (selectedService) params.set('service', selectedService);
      params.set('limit', '50');
      const resp = await fetch(`/api/otel/traces?${params}`);
      const data = await resp.json();
      setTraces(data);
    } catch (e: any) {
      console.error(e);
    }
    setLoading(false);
  };

  useEffect(() => { fetchServices(); }, []);
  useEffect(() => { fetchTraces(); }, [selectedService]);

  const handleViewTree = async (traceId: string) => {
    setTreeLoading(true);
    setSelectedTrace(traceId);
    try {
      const resp = await fetch(`/api/otel/traces/${traceId}`);
      const data = await resp.json();
      setTraceTree(data);
    } catch (e: any) {
      console.error(e);
    }
    setTreeLoading(false);
  };

  const statusColor = (status: string) => {
    switch (status) {
      case 'ok': case 'OK': return 'bg-green-100 text-green-700';
      case 'error': case 'ERROR': return 'bg-red-100 text-red-700';
      default: return 'bg-gray-100 text-gray-600';
    }
  };

  const renderTreeNode = (node: TraceTreeNode, depth: number = 0) => {
    const indent = depth * 20;
    const hasChildren = node.children.length > 0;
    return (
      <div key={node.span.span_id}>
        <div
          className="flex items-center gap-2 py-1.5 px-2 rounded-md hover:bg-md-surface-container/30"
          style={{ paddingLeft: `${indent + 8}px` }}
        >
          {hasChildren && <span className="text-xs text-md-on-surface-variant">▶</span>}
          {!hasChildren && <span className="text-xs text-md-on-surface-variant w-3" />}
          <span className={`px-1.5 py-0.5 rounded text-xs font-medium ${statusColor(node.span.status)}`}>
            {node.span.status}
          </span>
          <span className="text-sm text-md-on-surface font-mono">{node.span.operation_name}</span>
          <span className="text-xs text-md-on-surface-variant">({node.span.service})</span>
          <span className="text-xs text-md-on-surface-variant ml-auto">{node.span.duration_ms}ms</span>
        </div>
        {hasChildren && (
          <div>
            {node.children.map(child => renderTreeNode(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  const formatTime = (ts: number) => {
    return new Date(ts / 1000).toLocaleString();
  };

  return (
    <div className="space-y-6">
      {/* Filter area */}
      <div className="glass-card p-6">
        <h2 className="text-lg font-semibold text-md-on-surface mb-4">{t('trace_explorer.title')}</h2>
        <div className="flex gap-3">
          <select
            value={selectedService}
            onChange={e => setSelectedService(e.target.value)}
            className="px-4 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
          >
            <option value="">{t('trace_explorer.all_services')}</option>
            {services.map(s => (
              <option key={s} value={s}>{s}</option>
            ))}
          </select>
          <button
            onClick={fetchTraces}
            className="px-4 py-1.5 rounded-md-lg text-sm font-medium bg-md-surface-container text-md-on-surface hover:glass-card transition-all"
          >
            {t('trace_explorer.refresh')}
          </button>
        </div>
      </div>

      {/* Trace list */}
      <div className="glass-card p-6">
        {loading ? (
          <div className="text-center py-8 text-md-on-surface-variant">{t('trace_explorer.loading')}</div>
        ) : traces.length === 0 ? (
          <div className="text-center py-8 text-md-on-surface-variant">{t('trace_explorer.empty')}</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-md-outline-variant/50">
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">{t('trace_explorer.col_trace_id')}</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">{t('trace_explorer.col_operation')}</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">{t('trace_explorer.col_service')}</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">{t('trace_explorer.col_duration')}</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">{t('trace_explorer.col_spans')}</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">{t('trace_explorer.col_time')}</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">{t('trace_explorer.col_actions')}</th>
                </tr>
              </thead>
              <tbody>
                {traces.map(tr => (
                  <tr key={tr.trace_id} className="border-b border-md-outline-variant/20 hover:bg-md-surface-container/30">
                    <td className="px-4 py-2.5 font-mono text-xs text-md-on-surface">{tr.trace_id.slice(0, 12)}...</td>
                    <td className="px-4 py-2.5 text-md-on-surface">{tr.root_operation}</td>
                    <td className="px-4 py-2.5">
                      <span className="px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-700">
                        {tr.root_service}
                      </span>
                    </td>
                    <td className="px-4 py-2.5 text-md-on-surface font-mono">{tr.duration_ms}ms</td>
                    <td className="px-4 py-2.5 text-md-on-surface">{tr.span_count}</td>
                    <td className="px-4 py-2.5 text-xs text-md-on-surface-variant">{formatTime(tr.start_time)}</td>
                    <td className="px-4 py-2.5">
                      <button
                        onClick={() => handleViewTree(tr.trace_id)}
                        className="px-3 py-1 rounded-md text-xs font-medium bg-md-primary text-md-on-primary hover:opacity-90"
                      >
                        {t('trace_explorer.view_tree')}
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Trace tree view */}
      {selectedTrace && (
        <div className="glass-card p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-md font-semibold text-md-on-surface">
              {t('trace_explorer.tree_title')}: {selectedTrace.slice(0, 12)}...
            </h3>
            <button
              onClick={() => { setSelectedTrace(null); setTraceTree(null); }}
              className="px-3 py-1 rounded-md text-xs font-medium bg-md-surface-container text-md-on-surface hover:glass-card"
            >
              {t('trace_explorer.close')}
            </button>
          </div>

          {treeLoading ? (
            <div className="text-center py-8 text-md-on-surface-variant">{t('trace_explorer.loading')}</div>
          ) : !traceTree ? (
            <div className="text-center py-8 text-md-on-surface-variant">{t('trace_explorer.not_found')}</div>
          ) : (
            <div className="border border-md-outline-variant/30 rounded-md-lg overflow-hidden bg-md-surface/50">
              {renderTreeNode(traceTree)}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
