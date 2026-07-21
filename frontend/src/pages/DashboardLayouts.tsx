import { useState, useEffect } from 'react';

interface DashboardLayout {
  id: string;
  name: string;
  layout_json: string;
  created_at: string;
  updated_at: string;
}

export function DashboardLayoutsPage() {
  const [layouts, setLayouts] = useState<DashboardLayout[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState('');
  const [newJson, setNewJson] = useState('');
  const [editing, setEditing] = useState<DashboardLayout | null>(null);
  const [error, setError] = useState<string | null>(null);

  const fetchLayouts = async () => {
    setLoading(true);
    try {
      const resp = await fetch('/api/dashboard/layouts');
      const data = await resp.json();
      setLayouts(data);
    } catch (e: any) {
      setError(e.message);
    }
    setLoading(false);
  };

  useEffect(() => { fetchLayouts(); }, []);

  const handleCreate = async () => {
    if (!newName.trim() || !newJson.trim()) return;
    try {
      JSON.parse(newJson);
    } catch {
      setError('JSON 格式无效');
      return;
    }
    try {
      const resp = await fetch('/api/dashboard/layouts', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: newName, layout_json: newJson }),
      });
      if (!resp.ok) throw new Error('创建失败');
      setShowCreate(false);
      setNewName('');
      setNewJson('');
      fetchLayouts();
    } catch (e: any) {
      setError(e.message);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('确定删除此布局？')) return;
    try {
      await fetch(`/api/dashboard/layouts/${id}`, { method: 'DELETE' });
      fetchLayouts();
    } catch (e: any) {
      setError(e.message);
    }
  };

  const handleLoad = async (id: string) => {
    try {
      const resp = await fetch(`/api/dashboard/layouts/${id}`);
      const data = await resp.json();
      setEditing(data);
      setShowCreate(false);
    } catch (e: any) {
      setError(e.message);
    }
  };

  const handleSaveEdit = async () => {
    if (!editing) return;
    try {
      JSON.parse(editing.layout_json);
    } catch {
      setError('JSON 格式无效');
      return;
    }
    try {
      const resp = await fetch(`/api/dashboard/layouts/${editing.id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: editing.name, layout_json: editing.layout_json }),
      });
      if (!resp.ok) throw new Error('更新失败');
      setEditing(null);
      fetchLayouts();
    } catch (e: any) {
      setError(e.message);
    }
  };

  return (
    <div className="space-y-6">
      <div className="glass-card p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-md-on-surface">仪表盘布局管理</h2>
          <button
            onClick={() => { setShowCreate(!showCreate); setEditing(null); }}
            className="px-4 py-1.5 rounded-md-lg text-sm font-medium bg-md-primary text-md-on-primary hover:opacity-90 transition-all"
          >
            {showCreate ? '取消' : '+ 新建布局'}
          </button>
        </div>

        {error && (
          <div className="mb-4 p-3 rounded-md-lg bg-red-50 text-red-600 text-sm">{error}</div>
        )}

        {/* 新建表单 */}
        {showCreate && (
          <div className="mb-6 p-4 rounded-md-lg border border-md-outline-variant bg-md-surface-container/30 space-y-3">
            <input
              type="text"
              value={newName}
              onChange={e => setNewName(e.target.value)}
              placeholder="布局名称"
              className="w-full px-3 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
            />
            <textarea
              value={newJson}
              onChange={e => setNewJson(e.target.value)}
              placeholder='布局 JSON，例如: {"columns": 12, "widgets": []}'
              rows={6}
              className="w-full px-3 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm font-mono focus:outline-none focus:ring-2 focus:ring-md-primary/50"
            />
            <button
              onClick={handleCreate}
              className="px-4 py-1.5 rounded-md-lg text-sm font-medium bg-md-primary text-md-on-primary hover:opacity-90"
            >
              保存
            </button>
          </div>
        )}

        {/* 编辑表单 */}
        {editing && (
          <div className="mb-6 p-4 rounded-md-lg border border-md-primary bg-md-primary/5 space-y-3">
            <h3 className="text-sm font-medium text-md-on-surface">编辑: {editing.name}</h3>
            <input
              type="text"
              value={editing.name}
              onChange={e => setEditing({ ...editing, name: e.target.value })}
              className="w-full px-3 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
            />
            <textarea
              value={editing.layout_json}
              onChange={e => setEditing({ ...editing, layout_json: e.target.value })}
              rows={8}
              className="w-full px-3 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm font-mono focus:outline-none focus:ring-2 focus:ring-md-primary/50"
            />
            <div className="flex gap-2">
              <button onClick={handleSaveEdit} className="px-4 py-1.5 rounded-md-lg text-sm font-medium bg-md-primary text-md-on-primary hover:opacity-90">保存修改</button>
              <button onClick={() => setEditing(null)} className="px-4 py-1.5 rounded-md-lg text-sm font-medium bg-md-surface-container text-md-on-surface hover:glass-card">取消</button>
            </div>
          </div>
        )}

        {/* 布局列表 */}
        {loading ? (
          <div className="text-center py-8 text-md-on-surface-variant">加载中...</div>
        ) : layouts.length === 0 ? (
          <div className="text-center py-8 text-md-on-surface-variant">暂无布局，点击"新建布局"创建第一个</div>
        ) : (
          <div className="space-y-2">
            {layouts.map(layout => (
              <div key={layout.id} className="flex items-center justify-between p-3 rounded-md-lg bg-md-surface-container/30 border border-md-outline-variant/30">
                <div className="flex-1 min-w-0">
                  <h4 className="text-sm font-medium text-md-on-surface truncate">{layout.name}</h4>
                  <p className="text-xs text-md-on-surface-variant mt-0.5">
                    更新于 {layout.updated_at}
                  </p>
                </div>
                <div className="flex gap-2 ml-4">
                  <button onClick={() => handleLoad(layout.id)} className="px-3 py-1 rounded-md text-xs font-medium bg-md-surface-container text-md-on-surface hover:glass-card">编辑</button>
                  <button onClick={() => handleDelete(layout.id)} className="px-3 py-1 rounded-md text-xs font-medium bg-red-50 text-red-600 hover:bg-red-100">删除</button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
