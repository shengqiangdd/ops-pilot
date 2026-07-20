import { useEffect, useRef, useState, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import '@xterm/xterm/css/xterm.css';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error';

export function TerminalPage() {
  const { hostId } = useParams<{ hostId: string }>();
  const navigate = useNavigate();
  const { t } = useI18n();
  const { token } = useAuthStore();

  if (!hostId) {
    return (
      <div className="flex flex-col items-center justify-center h-[calc(100vh-8rem)] md:h-[calc(100vh-4rem)]">
        <div className="glass-card p-8 text-center space-y-4 max-w-md">
          <div className="text-4xl">⌨️</div>
          <h2 className="text-headline-small font-medium text-md-on-surface">{t('terminal.title')}</h2>
          <p className="text-body-medium text-md-on-surface-variant">{t('terminal.select_host_hint')}</p>
          <button
            onClick={() => navigate('/hosts')}
            className="text-sm px-4 py-2 rounded-md-full bg-md-primary text-md-on-primary hover:shadow-md-2 transition-all"
          >
            {t('terminal.go_to_hosts')}
          </button>
        </div>
      </div>
    );
  }

  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const [status, setStatus] = useState<ConnectionState>('connecting');
  const [errorMsg, setErrorMsg] = useState('');

  const buildWsUrl = useCallback(() => {
    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    return `${proto}//${window.location.host}/api/terminal/${hostId}?token=${token}`;
  }, [hostId, token]);

  const connect = useCallback(() => {
    if (!hostId || !token) return;

    setStatus('connecting');
    setErrorMsg('');

    const ws = new WebSocket(buildWsUrl());
    wsRef.current = ws;

    ws.onopen = () => {
      setStatus('connected');
      setErrorMsg('');
    };

    ws.onmessage = (ev) => {
      const term = xtermRef.current;
      if (!term) return;
      if (ev.data instanceof Blob) {
        ev.data.arrayBuffer().then((buf) => {
          term.write(new Uint8Array(buf));
        });
      } else {
        term.write(ev.data);
      }
    };

    ws.onerror = () => {
      setStatus('error');
      setErrorMsg(t('terminal.connection_error'));
    };

    ws.onclose = () => {
      wsRef.current = null;
      setStatus('disconnected');
      // auto-reconnect after 3s
      reconnectTimerRef.current = setTimeout(() => {
        if (!wsRef.current) connect();
      }, 3000);
    };
  }, [hostId, token, buildWsUrl, t]);

  // Init xterm + WebSocket
  useEffect(() => {
    if (!terminalRef.current) return;

    const term = new XTerm({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', Menlo, monospace",
      theme: {
        background: '#1C1B1F',
        foreground: '#E6E1E5',
        cursor: '#D0BCFF',
        selectionBackground: '#4F378B60',
      },
      allowProposedApi: true,
    });

    const fitAddon = new FitAddon();
    const webLinksAddon = new WebLinksAddon();

    term.loadAddon(fitAddon);
    term.loadAddon(webLinksAddon);
    term.open(terminalRef.current);

    xtermRef.current = term;
    fitAddonRef.current = fitAddon;

    // fit after open
    requestAnimationFrame(() => fitAddon.fit());

    // on user input → send to WS
    const onDataDisposable = term.onData((data) => {
      wsRef.current?.send(data);
    });

    // on resize → send resize message
    const onResizeDisposable = term.onResize(({ cols, rows }) => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ type: 'resize', cols, rows }));
      }
    });

    // window resize → fit
    const handleResize = () => {
      try { fitAddon.fit(); } catch { /* ignore */ }
    };
    window.addEventListener('resize', handleResize);

    // fit after container resize
    const resizeObserver = new ResizeObserver(() => {
      requestAnimationFrame(() => handleResize());
    });
    resizeObserver.observe(terminalRef.current);

    connect();

    return () => {
      onDataDisposable.dispose();
      onResizeDisposable.dispose();
      window.removeEventListener('resize', handleResize);
      resizeObserver.disconnect();
      if (reconnectTimerRef.current) clearTimeout(reconnectTimerRef.current);
      wsRef.current?.close();
      term.dispose();
      xtermRef.current = null;
      fitAddonRef.current = null;
    };
  }, [connect]);

  const handleDisconnect = () => {
    if (reconnectTimerRef.current) clearTimeout(reconnectTimerRef.current);
    wsRef.current?.close();
    wsRef.current = null;
    setStatus('disconnected');
  };

  const handleReconnect = () => {
    handleDisconnect();
    connect();
  };

  return (
    <div className="flex flex-col h-[calc(100vh-8rem)] md:h-[calc(100vh-4rem)]">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 bg-md-surface-container/70 backdrop-blur-xl border-b border-md-outline-variant/50 rounded-t-md-xl">
        <div className="flex items-center gap-3">
          <button
            onClick={() => navigate(-1)}
            className="flex items-center gap-1.5 text-sm text-md-on-surface-variant hover:text-md-on-surface transition-colors"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M15 19l-7-7 7-7" />
            </svg>
            {t('nav.back')}
          </button>
          <div className="w-px h-5 bg-md-outline-variant" />
          <span className="text-sm font-medium text-md-on-surface">
            {t('terminal.title')} — {hostId}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <span className={cn(
            'inline-flex items-center gap-1.5 text-xs font-medium px-2 py-1 rounded-full',
            status === 'connected' && 'bg-green-500/10 text-green-600 dark:text-green-400',
            status === 'connecting' && 'bg-amber-500/10 text-amber-600 dark:text-amber-400',
            status === 'disconnected' && 'bg-md-outline/10 text-md-on-surface-variant',
            status === 'error' && 'bg-md-error/10 text-md-error',
          )}>
            <span className={cn(
              'h-1.5 w-1.5 rounded-full',
              status === 'connected' && 'bg-green-500',
              status === 'connecting' && 'bg-amber-500 animate-pulse',
              status === 'disconnected' && 'bg-md-outline',
              status === 'error' && 'bg-md-error',
            )} />
            {t(`terminal.status.${status}`)}
          </span>
          {status === 'disconnected' || status === 'error' ? (
            <button
              onClick={handleReconnect}
              className="text-xs px-3 py-1.5 rounded-md-full bg-md-primary/10 text-md-primary hover:bg-md-primary/20 transition-colors font-medium"
            >
              {t('terminal.reconnect')}
            </button>
          ) : (
            <button
              onClick={handleDisconnect}
              className="text-xs px-3 py-1.5 rounded-md-full bg-md-error/10 text-md-error hover:bg-md-error/20 transition-colors font-medium"
            >
              {t('terminal.disconnect')}
            </button>
          )}
        </div>
      </div>

      {/* Terminal container */}
      <div className="flex-1 relative bg-[#1C1B1F] rounded-b-md-xl overflow-hidden">
        <div ref={terminalRef} className="absolute inset-0 p-1" />

        {/* Overlay states */}
        {status === 'connecting' && (
          <div className="absolute inset-0 flex items-center justify-center bg-[#1C1B1F]/80 backdrop-blur-sm z-10">
            <div className="flex flex-col items-center gap-3">
              <div className="h-8 w-8 border-2 border-md-primary border-t-transparent rounded-full animate-spin" />
              <span className="text-sm text-md-on-surface-variant">{t('terminal.connecting')}</span>
            </div>
          </div>
        )}

        {status === 'error' && errorMsg && (
          <div className="absolute inset-0 flex items-center justify-center bg-[#1C1B1F]/80 backdrop-blur-sm z-10">
            <div className="glass-card p-6 max-w-sm text-center space-y-3">
              <div className="text-3xl">⚠️</div>
              <p className="text-body-medium text-md-error font-medium">{errorMsg}</p>
              <button
                onClick={handleReconnect}
                className="text-sm px-4 py-2 rounded-md-full bg-md-primary text-md-on-primary hover:shadow-md-2 transition-all"
              >
                {t('terminal.reconnect')}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
