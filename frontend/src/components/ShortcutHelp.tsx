import { useState, useMemo, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import type { Shortcut } from '../hooks/useKeyboardShortcuts';
import { useI18n } from '../i18n';

interface Props {
  shortcuts: Shortcut[];
  open: boolean;
  onClose: () => void;
}

interface ShortcutGroup {
  category: string;
  items: Shortcut[];
}

function detectOS(): 'mac' | 'win' {
  if (typeof navigator === 'undefined') return 'win';
  return /Mac|iPod|iPhone|iPad/.test(navigator.platform) ? 'mac' : 'win';
}

function formatKeys(s: Shortcut, os: 'mac' | 'win'): string {
  const parts: string[] = [];
  if (os === 'mac') {
    if (s.ctrl) parts.push('⌘');
    if (s.alt) parts.push('⌥');
    if (s.shift) parts.push('⇧');
  } else {
    if (s.ctrl) parts.push('Ctrl');
    if (s.alt) parts.push('Alt');
    if (s.shift) parts.push('Shift');
  }
  parts.push(s.key.toUpperCase());
  return parts.join(os === 'mac' ? '' : '+');
}

const CATEGORY_LABELS: Record<string, string> = {
  navigation: '导航 Navigation',
  action: '操作 Actions',
  search: '搜索 Search',
};

function classifyShortcut(desc: string): 'navigation' | 'action' | 'search' {
  const lower = desc.toLowerCase();
  if (lower.includes('跳转') || lower.includes('go to') || lower.includes('navigate')) return 'navigation';
  if (lower.includes('搜索') || lower.includes('search') || lower.includes('命令') || lower.includes('command') || lower.includes('help')) return 'search';
  return 'action';
}

export function ShortcutHelp({ shortcuts, open, onClose }: Props) {
  const { t, lang } = useI18n();
  const [search, setSearch] = useState('');
  const os = useMemo(() => detectOS(), []);
  const inputRef = useRef<HTMLInputElement>(null);

  // Focus search input when panel opens
  useEffect(() => {
    if (open && inputRef.current) {
      setTimeout(() => inputRef.current?.focus(), 100);
    }
  }, [open]);

  // Reset search when panel closes
  useEffect(() => {
    if (!open) setSearch('');
  }, [open]);

  const grouped = useMemo(() => {
    const groups: Record<string, ShortcutGroup> = {};
    for (const s of shortcuts) {
      const cat = classifyShortcut(s.description);
      if (!groups[cat]) groups[cat] = { category: cat, items: [] };
      groups[cat].items.push(s);
    }
    return Object.values(groups);
  }, [shortcuts]);

  const filtered = useMemo(() => {
    if (!search.trim()) return grouped;
    const q = search.toLowerCase();
    return grouped
      .map(g => ({
        ...g,
        items: g.items.filter(s => s.description.toLowerCase().includes(q) || s.key.toLowerCase().includes(q)),
      }))
      .filter(g => g.items.length > 0);
  }, [grouped, search]);

  // Determine which shortcuts are "customizable" (action-type)
  const isCustomizable = (s: Shortcut) => classifyShortcut(s.description) === 'action';

  /* Escape key to close */
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [open, onClose]);

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          className="fixed inset-0 z-50 flex items-center justify-center"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.15 }}
        >
          {/* Backdrop */}
          <motion.div
            className="absolute inset-0 bg-black/40 backdrop-blur-sm"
            onClick={onClose}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          />

          {/* Panel */}
          <motion.div
            className="relative glass-card rounded-md-xl p-6 w-full max-w-lg mx-4 shadow-2xl max-h-[80vh] flex flex-col"
            onClick={(e) => e.stopPropagation()}
            initial={{ opacity: 0, scale: 0.92, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.92, y: 20 }}
            transition={{ duration: 0.2, ease: 'easeOut' }}
          >
            {/* Header */}
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-title-large text-md-on-surface font-semibold">
                {t('shortcut.help_title')}
              </h2>
              <button
                onClick={onClose}
                className="text-md-on-surface-variant hover:text-md-on-surface transition-colors p-1 rounded-md-lg hover:bg-md-surface-container-high"
                aria-label={t('nav.back')}
              >
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Search */}
            <div className="relative mb-4">
              <svg className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-md-on-surface-variant" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
              <input
                ref={inputRef}
                type="text"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder={lang === 'zh' ? '搜索快捷键...' : 'Search shortcuts...'}
                className="w-full bg-md-surface-container-highest/60 rounded-md-lg pl-10 pr-4 py-2.5 border border-md-outline-variant focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface transition-all duration-200"
              />
              {search && (
                <button
                  onClick={() => setSearch('')}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-md-on-surface-variant hover:text-md-on-surface"
                >
                  <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              )}
            </div>

            {/* Shortcuts list */}
            <div className="space-y-4 overflow-y-auto flex-1 -mx-2 px-2">
              {filtered.length === 0 ? (
                <p className="text-body-medium text-md-on-surface-variant text-center py-8">
                  {lang === 'zh' ? '没有匹配的快捷键' : 'No matching shortcuts'}
                </p>
              ) : (
                filtered.map((group) => (
                  <div key={group.category}>
                    <h3 className="text-label-medium text-md-on-surface-variant/70 uppercase tracking-wider mb-1.5 px-1">
                      {CATEGORY_LABELS[group.category] || group.category}
                    </h3>
                    <div className="space-y-0.5">
                      {group.items.map((s, i) => (
                        <div
                          key={i}
                          className="flex items-center justify-between py-2 px-3 rounded-md-lg hover:bg-md-surface-container-high transition-colors"
                        >
                          <span className="flex items-center gap-2 text-body-medium text-md-on-surface-variant">
                            {s.description}
                            {isCustomizable(s) && (
                              <span className="text-[10px] px-1.5 py-0.5 rounded-md-full bg-md-primary/10 text-md-primary font-medium">
                                {lang === 'zh' ? '可自定义' : 'Custom'}
                              </span>
                            )}
                          </span>
                          <kbd className="inline-flex items-center gap-1 px-2 py-1 text-xs font-mono rounded-md bg-md-surface-container-high text-md-on-surface-variant border border-md-outline-variant whitespace-nowrap">
                            {formatKeys(s, os)}
                          </kbd>
                        </div>
                      ))}
                    </div>
                  </div>
                ))
              )}
            </div>

            {/* Footer */}
            <p className="mt-4 text-body-small text-md-on-surface-variant text-center">
              {lang === 'zh'
                ? '按 Ctrl+K 或 Ctrl+/ 切换此面板 · 按 Esc 关闭'
                : 'Press Ctrl+K or Ctrl+/ to toggle · Press Esc to close'}
            </p>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
