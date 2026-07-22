import { useEffect, useRef, useState, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import '@xterm/xterm/css/xterm.css';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error';

interface TerminalTab {
  id: string;
  hostId: string;
  hostName: string;
  status: ConnectionState;
}

const DARK_THEME = {
  background: '#1C1B1F',
  foreground: '#E6E1E5',
  cursor: '#D0BCFF',
  selectionBackground: '#4F378B60',
};

const LIGHT_THEME = {
  background: '#FFFBFE',
  foreground: '#1C1B1F',
  cursor: '#6750A4',
  selectionBackground: '#6750A430',
};

export function TerminalPage() {
  const { hostId: urlHostId } = useParams<{ hostId: string }>();
  const { t } = useI18n();
  const { token } = useAuthStore();

  const [tabs, setTabs] = useState<TerminalTab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const [isDark, setIsDark] = useState(true);
  const [showNewConn, setShowNewConn] = useState(false);
  const [hosts, setHosts] = useState<Array<{ id: string; name: string }>>([]);

  const terminalRefs = useRef<Map<string, HTMLDivElement>>(new Map());
  const xtermRefs = useRef<Map<string, XTerm>>(new Map());
  const fitAddonRefs = useRef<Map<string, FitAddon>>(new Map());
  const wsRefs = useRef<Map<string, WebSocket>>(new Map());
  const reconnectTimers = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());
  const reconnectAttempts = useRef<Map<string, number>>(new Map());
  const MAX_RECONNECT_DELAY = 30000;
  const MAX_RECONNECT_ATTEMPTS = 10;

  // Load hosts list
  useEffect(() => {
    if (!token) return;
    import('../api/client').then(({ api }) => {
      api.listHosts(token).then((h) => setHosts(h.map((x) => ({ id: x.id, name: x.name })))).catch(() => {});
    });
  }, [token]);

  // Auto-create tab from URL param
  useEffect(() => {
    if (urlHostId && token && !tabs.find(t => t.hostId === urlHostId)) {
      const host = hosts.find(h => h.id === urlHostId);
      addTab(urlHostId, host?.name || urlHostId);
    }
  }, [urlHostId, token, hosts]);

  const buildWsUrl = useCallback((hid: string) => {
    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    return `${proto}//${window.location.host}/api/terminal/${hid}?token=${token}`;
  }, [token]);

  const connectTab = useCallback((tab: TerminalTab) => {
    if (!tab.hostId || !token) return;

    const ws = new WebSocket(buildWsUrl(tab.hostId));
    wsRefs.current.set(tab.id, ws);

    // Update status
    setTabs(prev => prev.map(t => t.id === tab.id ? { ...t, status: 'connecting' } : t));

    ws.onopen = () => {
      reconnectAttempts.current.set(tab.id, 0);
      setTabs(prev => prev.map(t => t.id === tab.id ? { ...t, status: 'connected' } : t));
    };

    ws.onmessage = (ev) => {
      const term = xtermRefs.current.get(tab.id);
      if (!term) return;
      if (ev.data instanceof Blob) {
        ev.data.arrayBuffer().then((buf) => term.write(new Uint8Array(buf)));
      } else {
        term.write(ev.data);
      }
    };

    ws.onerror = () => {
      setTabs(prev => prev.map(t => t.id === tab.id ? { ...t, status: 'error' } : t));
    };

    ws.onclose = () => {
      wsRefs.current.delete(tab.id);
      setTabs(prev => prev.map(t => t.id === tab.id ? { ...t, status: 'disconnected' } : t));

      const attempts = (reconnectAttempts.current.get(tab.id) || 0) + 1;
      reconnectAttempts.current.set(tab.id, attempts);

      if (attempts > MAX_RECONNECT_ATTEMPTS) {
        setTabs(prev => prev.map(t => t.id === tab.id ? { ...t, status: 'error' } : t));
        return;
      }

      // Exponential backoff: 1s, 2s, 4s, 8s, ... capped at MAX_RECONNECT_DELAY
      const delay = Math.min(1000 * Math.pow(2, attempts - 1), MAX_RECONNECT_DELAY);
      reconnectTimers.current.set(tab.id, setTimeout(() => {
        const currentTab = tabs.find(t => t.id === tab.id);
        if (currentTab && currentTab.status !== 'connected') {
          connectTab(currentTab);
        }
      }, delay));
    };

    wsRefs.current.set(tab.id, ws);
  }, [token, tabs, buildWsUrl]);

  const addTab = useCallback((hostId: string, hostName: string) => {
    const id = `tab-${Date.now()}`;
    const newTab: TerminalTab = { id, hostId, hostName, status: 'connecting' };
    setTabs(prev => [...prev, newTab]);
    setActiveTabId(id);

    // Init terminal after state update
    setTimeout(() => {
      const container = terminalRefs.current.get(id);
      if (!container) return;

      const theme = isDark ? DARK_THEME : LIGHT_THEME;
      const term = new XTerm({
        cursorBlink: true,
        fontSize: 14,
        fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', Menlo, monospace",
        theme,
        allowProposedApi: true,
      });

      const fitAddon = new FitAddon();
      const webLinksAddon = new WebLinksAddon();
      term.loadAddon(fitAddon);
      term.loadAddon(webLinksAddon);
      term.open(container);

      xtermRefs.current.set(id, term);
      fitAddonRefs.current.set(id, fitAddon);

      requestAnimationFrame(() => fitAddon.fit());

      // Send input to WS
      term.onData((data) => {
        wsRefs.current.get(id)?.send(data);
      });

      // Send resize
      term.onResize(({ cols, rows }) => {
        const ws = wsRefs.current.get(id);
        if (ws?.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify({ type: 'resize', cols, rows }));
        }
      });

      // Fit on window resize
      const handleResize = () => { try { fitAddon.fit(); } catch {} };
      window.addEventListener('resize', handleResize);

      // Fit on container resize
      const observer = new ResizeObserver(() => requestAnimationFrame(() => handleResize()));
      observer.observe(container);

      // Store cleanup
      const cleanup = () => {
        term.dispose();
        xtermRefs.current.delete(id);
        fitAddonRefs.current.delete(id);
        window.removeEventListener('resize', handleResize);
        observer.disconnect();
      };
      (container as any).__cleanup = cleanup;

      // Connect WebSocket
      const tab = tabs.find(t => t.id === id);
      if (tab) connectTab(tab);
    }, 100);
  }, [isDark, connectTab, tabs]);

  const removeTab = useCallback((id: string) => {
    // Cleanup terminal
    const container = terminalRefs.current.get(id);
    if (container && (container as any).__cleanup) {
      (container as any).__cleanup();
    }
    terminalRefs.current.delete(id);
    xtermRefs.current.delete(id);
    fitAddonRefs.current.delete(id);
    wsRefs.current.get(id)?.close();
    wsRefs.current.delete(id);
    if (reconnectTimers.current.has(id)) {
      clearTimeout(reconnectTimers.current.get(id)!);
      reconnectTimers.current.delete(id);
    }

    setTabs(prev => {
      const next = prev.filter(t => t.id !== id);
      if (activeTabId === id) {
        setActiveTabId(next.length > 0 ? next[next.length - 1].id : null);
      }
      return next;
    });
  }, [activeTabId]);

  const handleNewConnection = (hostId: string) => {
    const host = hosts.find(h => h.id === hostId);
    addTab(hostId, host?.name || hostId);
    setShowNewConn(false);
  };

  const toggleTheme = () => {
    setIsDark(!isDark);
    // Update all existing terminals
    const theme = !isDark ? DARK_THEME : LIGHT_THEME;
    xtermRefs.current.forEach((term) => {
      term.options.theme = theme;
    });
  };

  const handleCopy = () => {
    const term = activeTabId ? xtermRefs.current.get(activeTabId) : null;
    if (term) {
      const selection = term.getSelection();
      if (selection) {
        navigator.clipboard.writeText(selection);
      }
    }
  };

  const handlePaste = async () => {
    const term = activeTabId ? xtermRefs.current.get(activeTabId) : null;
    if (term) {
      const text = await navigator.clipboard.readText();
      term.write(text);
    }
  };

  // Empty state
  if (tabs.length === 0 && !showNewConn) {
    return (
      <div className="flex flex-col items-center justify-center h-[calc(100vh-8rem)] md:h-[calc(100vh-4rem)]">
        <div className="glass-card p-8 text-center space-y-4 max-w-md">
          <div className="text-4xl">⌨️</div>
          <h2 className="text-headline-small font-medium text-md-on-surface">{t('terminal.title')}</h2>
          <p className="text-body-medium text-md-on-surface-variant">{t('terminal.no_connections')}</p>
          <button
            onClick={() => setShowNewConn(true)}
            className="text-sm px-4 py-2 rounded-md-full bg-md-primary text-md-on-primary hover:shadow-md-2 transition-all"
          >
            {t('terminal.new_connection')}
          </button>
        </div>
      </div>
    );
  }

  const activeTab = tabs.find(t => t.id === activeTabId);

  return (
    <div className="flex flex-col h-[calc(100vh-8rem)] md:h-[calc(100vh-4rem)]">
      {/* Tab Bar */}
      <div className="flex items-center px-2 py-1 bg-md-surface-container/70 backdrop-blur-xl border-b border-md-outline-variant/50 rounded-t-md-xl overflow-x-auto">
        <div className="flex items-center gap-1">
          {tabs.map((tab) => (
            <div
              key={tab.id}
              onClick={() => setActiveTabId(tab.id)}
              className={cn(
                'flex items-center gap-2 px-3 py-1.5 rounded-md-lg cursor-pointer text-sm transition-colors',
                activeTabId === tab.id
                  ? 'bg-md-primary/10 text-md-primary'
                  : 'text-md-on-surface-variant hover:bg-md-surface-container-high',
              )}
            >
              <span className={cn(
                'h-1.5 w-1.5 rounded-full',
                tab.status === 'connected' && 'bg-green-500',
                tab.status === 'connecting' && 'bg-amber-500 animate-pulse',
                tab.status === 'disconnected' && 'bg-md-outline',
                tab.status === 'error' && 'bg-md-error',
              )} />
              <span className="max-w-[100px] truncate">{tab.hostName}</span>
              <button
                onClick={(e) => { e.stopPropagation(); removeTab(tab.id); }}
                className="text-xs hover:bg-md-error/20 rounded-full w-4 h-4 flex items-center justify-center"
              >
                ×
              </button>
            </div>
          ))}
          <button
            onClick={() => setShowNewConn(true)}
            className="px-2 py-1 text-sm text-md-on-surface-variant hover:text-md-primary hover:bg-md-surface-container-high rounded-md-lg transition-colors"
          >
            +
          </button>
        </div>

        <div className="flex-1" />

        {/* Toolbar */}
        <div className="flex items-center gap-1">
          <button onClick={handleCopy} className="px-2 py-1 text-xs text-md-on-surface-variant hover:bg-md-surface-container-high rounded-md-lg transition-colors" title="Copy">
            📋
          </button>
          <button onClick={handlePaste} className="px-2 py-1 text-xs text-md-on-surface-variant hover:bg-md-surface-container-high rounded-md-lg transition-colors" title="Paste">
            📄
          </button>
          <button onClick={toggleTheme} className="px-2 py-1 text-xs text-md-on-surface-variant hover:bg-md-surface-container-high rounded-md-lg transition-colors" title="Toggle Theme">
            {isDark ? '☀️' : '🌙'}
          </button>
        </div>
      </div>

      {/* Terminal Area */}
      <div className="flex-1 relative bg-[#1C1B1F] rounded-b-md-xl overflow-hidden">
        {tabs.map((tab) => (
          <div
            key={tab.id}
            ref={(el) => { if (el) terminalRefs.current.set(tab.id, el); }}
            className={cn('absolute inset-0 p-1', activeTabId === tab.id ? 'block' : 'hidden')}
          />
        ))}

        {/* Connecting overlay */}
        {activeTab && activeTab.status === 'connecting' && (
          <div className="absolute inset-0 flex items-center justify-center bg-[#1C1B1F]/80 backdrop-blur-sm z-10">
            <div className="flex flex-col items-center gap-3">
              <div className="h-8 w-8 border-2 border-md-primary border-t-transparent rounded-full animate-spin" />
              <span className="text-sm text-md-on-surface-variant">{t('terminal.connecting')}</span>
            </div>
          </div>
        )}

        {/* Disconnected with reconnection overlay */}
        {activeTab && activeTab.status === 'disconnected' && (
          <div className="absolute inset-0 flex items-center justify-center bg-[#1C1B1F]/80 backdrop-blur-sm z-10">
            <div className="flex flex-col items-center gap-3">
              <div className="h-8 w-8 border-2 border-amber-500 border-t-transparent rounded-full animate-spin" />
              <span className="text-sm text-amber-500">{t('terminal.reconnecting')}</span>
              <button
                onClick={() => {
                  const tab = tabs.find(t => t.id === activeTabId);
                  if (tab) {
                    wsRefs.current.get(tab.id)?.close();
                    connectTab(tab);
                  }
                }}
                className="text-xs px-3 py-1 rounded-md bg-md-surface-container text-md-on-surface hover:glass-card transition-all"
              >
                {t('terminal.reconnect')}
              </button>
            </div>
          </div>
        )}

        {/* Error overlay */}
        {activeTab && activeTab.status === 'error' && (
          <div className="absolute inset-0 flex items-center justify-center bg-[#1C1B1F]/80 backdrop-blur-sm z-10">
            <div className="glass-card p-6 max-w-sm text-center space-y-3">
              <div className="text-3xl">⚠️</div>
              <p className="text-body-medium text-md-error font-medium">{t('terminal.connection_error')}</p>
              <button
                onClick={() => {
                  const tab = tabs.find(t => t.id === activeTabId);
                  if (tab) {
                    wsRefs.current.get(tab.id)?.close();
                    connectTab(tab);
                  }
                }}
                className="text-sm px-4 py-2 rounded-md-full bg-md-primary text-md-on-primary hover:shadow-md-2 transition-all"
              >
                {t('terminal.reconnect')}
              </button>
            </div>
          </div>
        )}
      </div>

      {/* New Connection Dialog */}
      {showNewConn && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" onClick={() => setShowNewConn(false)}>
          <div className="glass-card rounded-md-2xl p-6 w-full max-w-md shadow-md-3 animate-scale-in" onClick={e => e.stopPropagation()}>
            <h3 className="text-title-large font-semibold text-md-on-surface mb-4">{t('terminal.new_connection')}</h3>
            <div className="space-y-3">
              <div>
                <label className="block text-label-large text-md-on-surface mb-1">{t('terminal.select_host')}</label>
                <select
                  onChange={(e) => { if (e.target.value) handleNewConnection(e.target.value); }}
                  className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface"
                >
                  <option value="">{t('terminal.select_host_hint')}</option>
                  {hosts.map(h => (
                    <option key={h.id} value={h.id}>{h.name}</option>
                  ))}
                </select>
              </div>
            </div>
            <div className="flex justify-end mt-4">
              <button
                onClick={() => setShowNewConn(false)}
                className="px-4 py-2 text-sm rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors"
              >
                {t('terminal.cancel')}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
