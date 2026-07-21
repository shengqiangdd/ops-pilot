import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

export interface Shortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  description: string;
  action: () => void;
}

export function useKeyboardShortcuts(shortcuts: Shortcut[]) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      // Ignore if user is typing in an input/textarea
      const tag = (e.target as HTMLElement)?.tagName?.toLowerCase();
      if (tag === 'input' || tag === 'textarea' || tag === 'select') return;

      for (const s of shortcuts) {
        const matchCtrl = s.ctrl ? (e.ctrlKey || e.metaKey) : true;
        const matchShift = s.shift ? e.shiftKey : true;
        const matchAlt = s.alt ? e.altKey : true;
        const matchKey = e.key.toLowerCase() === s.key.toLowerCase();

        if (matchKey && matchCtrl && matchShift && matchAlt) {
          e.preventDefault();
          e.stopPropagation();
          s.action();
          return;
        }
      }
    };

    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [shortcuts]);
}

/** Pre-built navigation shortcuts hook */
export function useNavigationShortcuts() {
  const navigate = useNavigate();
  const [showHelp, setShowHelp] = useState(false);

  const shortcuts: Shortcut[] = [
    {
      key: 'k',
      ctrl: true,
      description: '打开命令面板 (Cmd+K)',
      action: () => setShowHelp((p) => !p),
    },
    {
      key: '/',
      ctrl: true,
      description: '显示快捷键帮助',
      action: () => setShowHelp((p) => !p),
    },
    { key: 'd', ctrl: true, shift: true, description: '跳转到总览大屏', action: () => navigate('/ops-dashboard') },
    { key: 'h', ctrl: true, shift: true, description: '跳转到主机管理', action: () => navigate('/hosts') },
    { key: 'a', ctrl: true, shift: true, description: '跳转到告警', action: () => navigate('/alert-history') },
    { key: 't', ctrl: true, shift: true, description: '跳转到终端', action: () => navigate('/terminal') },
    { key: 'u', ctrl: true, shift: true, description: '跳转到用户管理', action: () => navigate('/users') },
    { key: 'i', ctrl: true, shift: true, description: '跳转到智能巡检', action: () => navigate('/inspection') },
    { key: 'm', ctrl: true, shift: true, description: '跳转到监控面板', action: () => navigate('/monitor') },
  ];

  return { shortcuts, showHelp, setShowHelp };
}
