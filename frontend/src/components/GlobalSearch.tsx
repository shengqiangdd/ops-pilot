import { useState, useRef, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';

interface SearchResult {
  type: 'host' | 'alert' | 'runbook' | 'knowledge' | 'page';
  label: string;
  description?: string;
  path: string;
}

const PAGES = [
  { label: '总览大屏', path: '/ops-dashboard', type: 'page' as const },
  { label: '主机列表', path: '/hosts', type: 'page' as const },
  { label: '监控大盘', path: '/health', type: 'page' as const },
  { label: '告警规则', path: '/alert-rules', type: 'page' as const },
  { label: '告警历史', path: '/alert-history', type: 'page' as const },
  { label: '通知渠道', path: '/channels', type: 'page' as const },
  { label: '安全合规', path: '/security', type: 'page' as const },
  { label: '运维报告', path: '/reports', type: 'page' as const },
  { label: '知识库', path: '/knowledge', type: 'page' as const },
  { label: '备份恢复', path: '/backup', type: 'page' as const },
  { label: '配置管理', path: '/config', type: 'page' as const },
  { label: 'API 文档', path: '/api-docs', type: 'page' as const },
  { label: '任务调度', path: '/jobs', type: 'page' as const },
  { label: 'Webhook', path: '/webhook', type: 'page' as const },
  { label: '智能分析', path: '/diagnostics', type: 'page' as const },
  { label: '费用分析', path: '/finops', type: 'page' as const },
];

export function GlobalSearch() {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();

  // Cmd+K / Ctrl+K toggle
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setOpen(prev => !prev);
      }
      if (e.key === 'Escape') setOpen(false);
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  useEffect(() => {
    if (open && inputRef.current) {
      inputRef.current.focus();
    }
  }, [open]);

  const doSearch = useCallback(async (q: string) => {
    if (!q.trim()) {
      setResults(PAGES.map(p => ({ ...p, label: p.label })).slice(0, 8));
      return;
    }
    const lower = q.toLowerCase();

    // Local page search
    const pageResults = PAGES
      .filter(p => p.label.toLowerCase().includes(lower))
      .map(p => ({ type: 'page' as const, label: p.label, path: p.path }));

    // Try to fetch from API
    let apiResults: SearchResult[] = [];
    try {
      const resp = await fetch(`/api/search?q=${encodeURIComponent(q)}`);
      if (resp.ok) {
        const data = await resp.json();
        apiResults = (data.results || []).map((r: any) => ({
          type: r.type,
          label: r.label,
          description: r.description,
          path: r.path,
        }));
      }
    } catch { /* ignore */ }

    setResults([...pageResults, ...apiResults].slice(0, 10));
    setSelectedIndex(0);
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => doSearch(query), 150);
    return () => clearTimeout(timer);
  }, [query, doSearch]);

  const handleSelect = (result: SearchResult) => {
    setOpen(false);
    setQuery('');
    navigate(result.path);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex(i => Math.min(i + 1, results.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex(i => Math.max(i - 1, 0));
    } else if (e.key === 'Enter' && results[selectedIndex]) {
      handleSelect(results[selectedIndex]);
    }
  };

  return (
    <>
      {/* Search trigger button in header */}
      <button
        onClick={() => setOpen(true)}
        className="flex items-center gap-2 px-3 py-1.5 text-sm text-md-on-surface-variant bg-md-surface-container-high/50 border border-md-outline-variant/50 rounded-md-lg hover:bg-md-surface-container-high transition-all min-w-[200px]"
      >
        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <span className="flex-1 text-left">搜索...</span>
        <kbd className="text-[10px] px-1.5 py-0.5 rounded bg-md-surface-variant/50 text-md-on-surface-variant font-mono">⌘K</kbd>
      </button>

      <AnimatePresence>
        {open && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/40 backdrop-blur-sm"
            onClick={() => setOpen(false)}
          >
            <motion.div
              initial={{ scale: 0.95, opacity: 0, y: -20 }}
              animate={{ scale: 1, opacity: 1, y: 0 }}
              exit={{ scale: 0.95, opacity: 0, y: -20 }}
              className="w-full max-w-xl bg-md-surface-container-high/95 backdrop-blur-2xl rounded-md-xl shadow-2xl border border-md-outline-variant/50 overflow-hidden"
              onClick={e => e.stopPropagation()}
            >
              <div className="flex items-center gap-3 px-4 py-3 border-b border-md-outline-variant/30">
                <svg className="w-5 h-5 text-md-on-surface-variant shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
                <input
                  ref={inputRef}
                  type="text"
                  value={query}
                  onChange={e => setQuery(e.target.value)}
                  onKeyDown={handleKeyDown}
                  placeholder="搜索页面、主机、告警、知识库..."
                  className="flex-1 bg-transparent border-none outline-none text-md-on-surface placeholder:text-md-on-surface-variant/50 text-base"
                />
                <kbd className="text-[10px] px-1.5 py-0.5 rounded bg-md-surface-variant/50 text-md-on-surface-variant font-mono">ESC</kbd>
              </div>

              {/* Results */}
              <div className="max-h-[400px] overflow-y-auto p-2">
                {results.length === 0 && (
                  <div className="py-8 text-center text-md-on-surface-variant/50 text-sm">无结果</div>
                )}
                {results.map((r, i) => (
                  <button
                    key={`${r.type}-${r.label}-${i}`}
                    onClick={() => handleSelect(r)}
                    className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-md-lg text-left transition-colors ${
                      i === selectedIndex
                        ? 'bg-md-primary/15 text-md-on-surface'
                        : 'text-md-on-surface-variant hover:bg-md-surface-variant/30'
                    }`}
                  >
                    <span className="text-lg shrink-0">
                      {r.type === 'host' ? '🖥️' : r.type === 'alert' ? '🔔' : r.type === 'runbook' ? '📋' : r.type === 'knowledge' ? '📚' : '📄'}
                    </span>
                    <div className="flex-1 min-w-0">
                      <div className="text-sm font-medium truncate">{r.label}</div>
                      {r.description && (
                        <div className="text-xs text-md-on-surface-variant/60 truncate">{r.description}</div>
                      )}
                    </div>
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-md-surface-variant/30 text-md-on-surface-variant/50 uppercase">{r.type}</span>
                  </button>
                ))}
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}
