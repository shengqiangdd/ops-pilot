/**
 * Global WebSocket connection status manager.
 *
 * Provides:
 * - A singleton WebSocket connection to the event bus endpoint
 * - Connection status: connected | connecting | disconnected
 * - Auto-reconnect with exponential backoff (1s → 2s → 4s → … → max 30s)
 * - A callback to register reconnections
 */

import { useEffect, useRef, useState, useCallback } from 'react';
import { useAuthStore } from '../stores/useAuthStore';

export type WSStatus = 'connected' | 'connecting' | 'disconnected';

const RECONNECT_BASE = 1000; // 1s
const RECONNECT_MAX = 30000; // 30s
const WS_PATH = '/api/ws/events';

let globalStatusListeners: Set<(s: WSStatus) => void> = new Set();
let globalWsRef: WebSocket | null = null;
let globalStatus: WSStatus = 'disconnected';
let globalReconnectTimer: ReturnType<typeof setTimeout> | null = null;
let globalAttempts = 0;

function notifyListeners(status: WSStatus) {
  globalStatus = status;
  globalStatusListeners.forEach(fn => fn(status));
}

function scheduleReconnect(getToken: () => string | null) {
  if (globalReconnectTimer) {
    clearTimeout(globalReconnectTimer);
  }
  globalAttempts++;
  const delay = Math.min(RECONNECT_BASE * Math.pow(2, globalAttempts - 1), RECONNECT_MAX);
  globalReconnectTimer = setTimeout(() => {
    connect(getToken);
  }, delay);
}

function cleanup() {
  if (globalReconnectTimer) {
    clearTimeout(globalReconnectTimer);
    globalReconnectTimer = null;
  }
  if (globalWsRef) {
    globalWsRef.onopen = null;
    globalWsRef.onclose = null;
    globalWsRef.onerror = null;
    globalWsRef.onmessage = null;
    if (globalWsRef.readyState === WebSocket.OPEN || globalWsRef.readyState === WebSocket.CONNECTING) {
      globalWsRef.close();
    }
    globalWsRef = null;
  }
}

function connect(getToken: () => string | null) {
  const token = getToken();
  if (!token) {
    // No token, wait for auth
    notifyListeners('disconnected');
    scheduleReconnect(getToken);
    return;
  }

  cleanup();

  const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const url = `${proto}//${window.location.host}${WS_PATH}?token=${token}`;

  notifyListeners('connecting');
  try {
    const ws = new WebSocket(url);
    globalWsRef = ws;

    ws.onopen = () => {
      globalAttempts = 0;
      notifyListeners('connected');
    };

    ws.onclose = () => {
      globalWsRef = null;
      notifyListeners('disconnected');
      scheduleReconnect(getToken);
    };

    ws.onerror = () => {
      // onclose will fire after onerror, which handles reconnect
    };
  } catch {
    notifyListeners('disconnected');
    scheduleReconnect(getToken);
  }
}

/**
 * Hook that provides the current global WebSocket status and establishes
 * the connection if the user is authenticated.
 */
export function useWebSocketStatus(): WSStatus {
  const [status, setStatus] = useState<WSStatus>(globalStatus);
  const { token } = useAuthStore();
  const tokenRef = useRef(token);

  tokenRef.current = token;

  const getToken = useCallback(() => tokenRef.current, []);

  useEffect(() => {
    // Register listener
    globalStatusListeners.add(setStatus);

    // Start or restart connection if we have a token
    if (token && (!globalWsRef || globalWsRef.readyState === WebSocket.CLOSED)) {
      connect(getToken);
    }

    return () => {
      globalStatusListeners.delete(setStatus);
    };
  }, [token, getToken]);

  return status;
}

/**
 * Get the current global status without subscribing.
 */
export function getCurrentWSStatus(): WSStatus {
  return globalStatus;
}

/**
 * Initialize / reconnect the global WebSocket manually.
 */
export function reconnectWS() {
  globalAttempts = 0;
  cleanup();
  const token = useAuthStore.getState().token;
  if (token) {
    connect(() => token);
  }
}
