import { useState } from 'react';

export function BackupRestorePage() {
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<string | null>(null);

  const handleExport = async () => {
    setLoading(true);
    setResult(null);
    try {
      const resp = await fetch('/api/backup/export');
      const data = await resp.json();
      if (data.status === 'ok') {
        // Download as file
        const blob = new Blob([JSON.stringify(data.data, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `ops-pilot-backup-${new Date().toISOString().slice(0, 10)}.json`;
        a.click();
        URL.revokeObjectURL(url);
        setResult('✅ 备份已下载');
      } else {
        setResult(`❌ ${data.message}`);
      }
    } catch (e: any) {
      setResult(`❌ ${e.message}`);
    }
    setLoading(false);
  };

  const handleImport = async () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      setLoading(true);
      setResult(null);
      try {
        const text = await file.text();
        const backup = JSON.parse(text);
        const resp = await fetch('/api/backup/import', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(backup),
        });
        const data = await resp.json();
        if (data.status === 'ok') {
          setResult(`✅ 恢复成功: ${data.results?.join(', ') || ''}`);
        } else {
          setResult(`❌ ${data.message}`);
        }
      } catch (e: any) {
        setResult(`❌ ${e.message}`);
      }
      setLoading(false);
    };
    input.click();
  };

  return (
    <div className="p-6 max-w-2xl mx-auto space-y-8">
      <div>
        <h1 className="text-2xl font-bold text-md-on-surface mb-2">🔄 备份与恢复</h1>
        <p className="text-md-on-surface-variant">导出所有配置（用户、通知渠道、告警规则、runbook 等）为 JSON 文件，或从备份文件恢复系统。</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Export Card */}
        <div className="glass-card p-6 rounded-md-xl space-y-4">
          <div className="flex items-center gap-3">
            <span className="text-3xl">📤</span>
            <div>
              <h2 className="text-lg font-semibold text-md-on-surface">导出备份</h2>
              <p className="text-sm text-md-on-surface-variant">下载当前全部配置</p>
            </div>
          </div>
          <button
            onClick={handleExport}
            disabled={loading}
            className="w-full px-4 py-3 bg-md-primary text-md-on-primary rounded-md-lg hover:opacity-90 disabled:opacity-50 font-medium transition-all"
          >
            {loading ? '处理中...' : '📥 导出 JSON'}
          </button>
        </div>

        {/* Import Card */}
        <div className="glass-card p-6 rounded-md-xl space-y-4">
          <div className="flex items-center gap-3">
            <span className="text-3xl">📥</span>
            <div>
              <h2 className="text-lg font-semibold text-md-on-surface">导入备份</h2>
              <p className="text-sm text-md-on-surface-variant">从文件恢复系统配置</p>
            </div>
          </div>
          <button
            onClick={handleImport}
            disabled={loading}
            className="w-full px-4 py-3 bg-md-secondary text-md-on-secondary rounded-md-lg hover:opacity-90 disabled:opacity-50 font-medium transition-all"
          >
            {loading ? '处理中...' : '📄 选择文件恢复'}
          </button>
        </div>
      </div>

      {result && (
        <div className={`p-4 rounded-md-lg text-sm font-medium ${
          result.startsWith('✅') ? 'bg-green-500/10 text-green-600 border border-green-500/20' : 'bg-red-500/10 text-red-600 border border-red-500/20'
        }`}>
          {result}
        </div>
      )}
    </div>
  );
}
