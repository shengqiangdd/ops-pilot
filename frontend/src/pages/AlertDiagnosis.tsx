import { useState } from 'react';

interface KnowledgeMatch {
  id: string;
  title: string;
  root_cause: string;
  resolution: string;
}

interface DiagnosisResult {
  alert: {
    id: string;
    message: string;
    severity: string;
    resource: string;
    created_at: string;
  };
  rule_analysis: string;
  suggestions: string[];
  knowledge_matches: KnowledgeMatch[];
}

export function AlertDiagnosisPage() {
  const [alertId, setAlertId] = useState('');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<DiagnosisResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleDiagnose = async () => {
    if (!alertId.trim()) return;
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const resp = await fetch(`/api/alert/diagnose/${alertId}`);
      if (!resp.ok) {
        const data = await resp.json();
        throw new Error(data.error || '诊断失败');
      }
      const data = await resp.json();
      setResult(data);
    } catch (e: any) {
      setError(e.message);
    }
    setLoading(false);
  };

  const severityColor = (s: string) => {
    switch (s) {
      case 'critical': return 'bg-red-100 text-red-700';
      case 'warning': return 'bg-amber-100 text-amber-700';
      default: return 'bg-blue-100 text-blue-700';
    }
  };

  return (
    <div className="space-y-6">
      {/* 输入区域 */}
      <div className="glass-card p-6">
        <h2 className="text-lg font-semibold text-md-on-surface mb-4">AI 告警诊断</h2>
        <p className="text-sm text-md-on-surface-variant mb-4">
          输入告警 ID，系统将自动分析告警原因、匹配知识库并生成处置建议。
        </p>
        <div className="flex gap-3">
          <input
            type="text"
            value={alertId}
            onChange={e => setAlertId(e.target.value)}
            placeholder="输入告警 ID..."
            className="flex-1 px-4 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface placeholder:text-md-on-surface-variant/50 focus:outline-none focus:ring-2 focus:ring-md-primary/50"
            onKeyDown={e => e.key === 'Enter' && handleDiagnose()}
          />
          <button
            onClick={handleDiagnose}
            disabled={loading || !alertId.trim()}
            className="px-6 py-2 rounded-md-lg bg-md-primary text-md-on-primary font-medium hover:opacity-90 disabled:opacity-50 transition-all"
          >
            {loading ? '诊断中...' : '开始诊断'}
          </button>
        </div>
      </div>

      {/* 错误提示 */}
      {error && (
        <div className="glass-card p-4 border-l-4 border-red-500">
          <p className="text-red-600 text-sm">❌ {error}</p>
        </div>
      )}

      {/* 诊断结果 */}
      {result && (
        <div className="space-y-4">
          {/* 告警信息 */}
          <div className="glass-card p-6">
            <h3 className="text-md font-semibold text-md-on-surface mb-3">告警信息</h3>
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span className="text-md-on-surface-variant">ID:</span>
                <span className="ml-2 text-md-on-surface font-mono">{result.alert.id}</span>
              </div>
              <div>
                <span className="text-md-on-surface-variant">严重级别:</span>
                <span className={`ml-2 px-2 py-0.5 rounded-full text-xs font-medium ${severityColor(result.alert.severity)}`}>
                  {result.alert.severity}
                </span>
              </div>
              <div>
                <span className="text-md-on-surface-variant">资源:</span>
                <span className="ml-2 text-md-on-surface">{result.alert.resource}</span>
              </div>
              <div>
                <span className="text-md-on-surface-variant">时间:</span>
                <span className="ml-2 text-md-on-surface">{result.alert.created_at}</span>
              </div>
              <div className="col-span-2">
                <span className="text-md-on-surface-variant">消息:</span>
                <span className="ml-2 text-md-on-surface">{result.alert.message}</span>
              </div>
            </div>
          </div>

          {/* 规则分析 */}
          <div className="glass-card p-6">
            <h3 className="text-md font-semibold text-md-on-surface mb-3">规则分析</h3>
            <p className="text-sm text-md-on-surface leading-relaxed">{result.rule_analysis}</p>
          </div>

          {/* 处置建议 */}
          <div className="glass-card p-6">
            <h3 className="text-md font-semibold text-md-on-surface mb-3">处置建议</h3>
            <ul className="space-y-2">
              {result.suggestions.map((s, i) => (
                <li key={i} className="flex items-start gap-2 text-sm text-md-on-surface">
                  <span className="text-md-primary mt-0.5">•</span>
                  <span>{s}</span>
                </li>
              ))}
            </ul>
          </div>

          {/* 知识库匹配 */}
          {result.knowledge_matches.length > 0 && (
            <div className="glass-card p-6">
              <h3 className="text-md font-semibold text-md-on-surface mb-3">相关知识库条目</h3>
              <div className="space-y-3">
                {result.knowledge_matches.map(k => (
                  <div key={k.id} className="p-3 rounded-md-lg bg-md-surface-container/50 border border-md-outline-variant/30">
                    <h4 className="text-sm font-medium text-md-on-surface">{k.title}</h4>
                    <p className="text-xs text-md-on-surface-variant mt-1">根因: {k.root_cause}</p>
                    <p className="text-xs text-md-on-surface-variant mt-1">解决方案: {k.resolution}</p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
