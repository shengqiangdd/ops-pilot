import { useState } from 'react';

interface CorrelatedEvent {
  id: string;
  event_type: string;
  message: string;
  severity: string;
  resource: string;
  created_at: string;
  relevance_score: number;
}

interface RootCause {
  cause: string;
  score: number;
  evidence: string[];
}

interface CorrelationResult {
  alert_id: string;
  correlated_events: CorrelatedEvent[];
  root_causes: RootCause[];
}

interface CausalChainEvent {
  id: string;
  event_type: string;
  message: string;
  resource: string;
  created_at: string;
  sequence: number;
}

interface CausalChainResult {
  incident_id: string;
  chain: CausalChainEvent[];
  summary: string;
}

export function RCAnalysisPage() {
  const [inputId, setInputId] = useState('');
  const [mode, setMode] = useState<'correlate' | 'causal'>('correlate');
  const [correlation, setCorrelation] = useState<CorrelationResult | null>(null);
  const [causalChain, setCausalChain] = useState<CausalChainResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleAnalyze = async () => {
    if (!inputId.trim()) return;
    setLoading(true);
    setError(null);
    setCorrelation(null);
    setCausalChain(null);

    try {
      if (mode === 'correlate') {
        const resp = await fetch(`/api/rca/correlate/${inputId}`);
        if (!resp.ok) {
          const data = await resp.json();
          throw new Error(data.error || '分析失败');
        }
        setCorrelation(await resp.json());
      } else {
        const resp = await fetch(`/api/rca/causal-chain/${inputId}`);
        if (!resp.ok) {
          const data = await resp.json();
          throw new Error(data.error || '分析失败');
        }
        setCausalChain(await resp.json());
      }
    } catch (e: any) {
      setError(e.message);
    }
    setLoading(false);
  };

  const eventTypeColor = (type: string) => {
    switch (type) {
      case 'alert': return 'bg-red-100 text-red-700';
      case 'audit': return 'bg-blue-100 text-blue-700';
      case 'trigger': return 'bg-amber-100 text-amber-700';
      case 'related': return 'bg-gray-100 text-gray-700';
      default: return 'bg-gray-100 text-gray-600';
    }
  };

  const scoreColor = (score: number) => {
    if (score >= 0.8) return 'text-red-600';
    if (score >= 0.6) return 'text-amber-600';
    return 'text-green-600';
  };

  return (
    <div className="space-y-6">
      {/* 输入区域 */}
      <div className="glass-card p-6">
        <h2 className="text-lg font-semibold text-md-on-surface mb-4">智能根因定位</h2>
        <div className="flex gap-3 mb-4">
          <div className="flex rounded-md-lg border border-md-outline-variant overflow-hidden">
            <button
              onClick={() => setMode('correlate')}
              className={`px-4 py-2 text-sm font-medium transition-all ${
                mode === 'correlate'
                  ? 'bg-md-primary text-md-on-primary'
                  : 'bg-md-surface text-md-on-surface hover:bg-md-surface-container'
              }`}
            >
              关联分析
            </button>
            <button
              onClick={() => setMode('causal')}
              className={`px-4 py-2 text-sm font-medium transition-all ${
                mode === 'causal'
                  ? 'bg-md-primary text-md-on-primary'
                  : 'bg-md-surface text-md-on-surface hover:bg-md-surface-container'
              }`}
            >
              因果链
            </button>
          </div>
          <input
            type="text"
            value={inputId}
            onChange={e => setInputId(e.target.value)}
            placeholder={mode === 'correlate' ? '输入告警 ID...' : '输入事故 ID...'}
            className="flex-1 px-4 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface placeholder:text-md-on-surface-variant/50 focus:outline-none focus:ring-2 focus:ring-md-primary/50"
            onKeyDown={e => e.key === 'Enter' && handleAnalyze()}
          />
          <button
            onClick={handleAnalyze}
            disabled={loading || !inputId.trim()}
            className="px-6 py-2 rounded-md-lg bg-md-primary text-md-on-primary font-medium hover:opacity-90 disabled:opacity-50 transition-all"
          >
            {loading ? '分析中...' : '开始分析'}
          </button>
        </div>
        {error && (
          <div className="p-3 rounded-md-lg bg-red-50 text-red-600 text-sm">❌ {error}</div>
        )}
      </div>

      {/* 关联分析结果 */}
      {correlation && (
        <div className="space-y-4">
          {/* 根因评分 */}
          <div className="glass-card p-6">
            <h3 className="text-md font-semibold text-md-on-surface mb-3">根因评分排名</h3>
            {correlation.root_causes.length === 0 ? (
              <p className="text-sm text-md-on-surface-variant">未发现明确根因</p>
            ) : (
              <div className="space-y-3">
                {correlation.root_causes.map((rc, i) => (
                  <div key={i} className="p-3 rounded-md-lg bg-md-surface-container/30 border border-md-outline-variant/30">
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-sm font-medium text-md-on-surface">{rc.cause}</span>
                      <span className={`text-lg font-bold ${scoreColor(rc.score)}`}>
                        {(rc.score * 100).toFixed(0)}%
                      </span>
                    </div>
                    {rc.evidence.length > 0 && (
                      <div className="mt-2">
                        <span className="text-xs text-md-on-surface-variant">证据:</span>
                        <ul className="mt-1 space-y-0.5">
                          {rc.evidence.map((e, j) => (
                            <li key={j} className="text-xs text-md-on-surface-variant pl-2 border-l-2 border-md-outline-variant/30">
                              {e}
                            </li>
                          ))}
                        </ul>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* 关联事件 */}
          <div className="glass-card p-6">
            <h3 className="text-md font-semibold text-md-on-surface mb-3">
              关联事件 ({correlation.correlated_events.length})
            </h3>
            <div className="space-y-2">
              {correlation.correlated_events.map(event => (
                <div key={event.id} className="flex items-start gap-3 p-3 rounded-md-lg bg-md-surface-container/20 border border-md-outline-variant/20">
                  <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${eventTypeColor(event.event_type)}`}>
                    {event.event_type}
                  </span>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm text-md-on-surface">{event.message}</p>
                    <p className="text-xs text-md-on-surface-variant mt-0.5">
                      {event.resource} · {event.created_at}
                    </p>
                  </div>
                  <span className="text-xs text-md-on-surface-variant">
                    相关度: {(event.relevance_score * 100).toFixed(0)}%
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* 因果链结果 */}
      {causalChain && (
        <div className="glass-card p-6">
          <h3 className="text-md font-semibold text-md-on-surface mb-2">因果链时间线</h3>
          <p className="text-sm text-md-on-surface-variant mb-4">{causalChain.summary}</p>
          <div className="relative">
            {/* 时间线 */}
            <div className="absolute left-4 top-0 bottom-0 w-0.5 bg-md-outline-variant/50" />
            <div className="space-y-4">
              {causalChain.chain.map((event) => (
                <div key={event.id} className="relative flex items-start gap-4 pl-10">
                  {/* 节点 */}
                  <div className={`absolute left-2.5 w-3 h-3 rounded-full border-2 ${
                    event.event_type === 'trigger'
                      ? 'bg-red-500 border-red-500'
                      : 'bg-md-surface border-md-outline-variant'
                  }`} />
                  <div className="flex-1 p-3 rounded-md-lg bg-md-surface-container/20 border border-md-outline-variant/20">
                    <div className="flex items-center gap-2 mb-1">
                      <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${eventTypeColor(event.event_type)}`}>
                        {event.event_type}
                      </span>
                      <span className="text-xs text-md-on-surface-variant font-mono">{event.created_at}</span>
                    </div>
                    <p className="text-sm text-md-on-surface">{event.message}</p>
                    <p className="text-xs text-md-on-surface-variant mt-0.5">资源: {event.resource}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
