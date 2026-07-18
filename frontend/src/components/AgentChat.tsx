import { useCallback, useEffect, useRef, useState } from 'react';
import { api } from '../api/client';
import type { AgentResponse } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  turns?: AgentResponse['turns'];
}

export function AgentChat() {
  const token = useAuthStore((s) => s.token);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [sending, setSending] = useState(false);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const bottomRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // Create session on mount
  useEffect(() => {
    if (!token) return;
    let cancelled = false;
    api
      .createAgentSession(token)
      .then((s) => {
        if (!cancelled) setSessionId(s.session_id);
      })
      .catch((e) => {
        if (!cancelled) setError(e instanceof Error ? e.message : 'Failed to create session');
      });
    return () => {
      cancelled = true;
    };
  }, [token]);

  const sendMessage = useCallback(async () => {
    const text = input.trim();
    if (!text || !sessionId || !token || sending) return;

    setInput('');
    setError(null);

    const userMsg: ChatMessage = {
      id: `user-${Date.now()}`,
      role: 'user',
      content: text,
    };
    setMessages((prev) => [...prev, userMsg]);
    setSending(true);

    try {
      const resp: AgentResponse = await api.sendAgentMessage(token, sessionId, text);
      const assistantMsg: ChatMessage = {
        id: `assistant-${Date.now()}`,
        role: 'assistant',
        content: resp.content,
        turns: resp.turns,
      };
      setMessages((prev) => [...prev, assistantMsg]);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to send message');
    } finally {
      setSending(false);
    }
  }, [input, sessionId, token, sending]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  if (!token) {
    return (
      <div className="rounded-lg border border-gray-200 bg-white p-8 text-center text-gray-500">
        Please log in to use the agent chat.
      </div>
    );
  }

  return (
    <div className="flex flex-col rounded-lg border border-gray-200 bg-white" style={{ height: 'calc(100vh - 12rem)' }}>
      {/* Messages area */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {messages.length === 0 && (
          <div className="flex h-full items-center justify-center text-sm text-gray-400">
            Start a conversation with the OpsPilot agent.
          </div>
        )}
        {messages.map((msg) => (
          <div
            key={msg.id}
            className={cn('flex', msg.role === 'user' ? 'justify-end' : 'justify-start')}
          >
            <div
              className={cn(
                'max-w-[75%] rounded-lg px-4 py-2.5 text-sm leading-relaxed',
                msg.role === 'user'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-900',
              )}
            >
              <div className="whitespace-pre-wrap">{msg.content}</div>
              {msg.turns && msg.turns.length > 0 && (
                <div className="mt-2 border-t border-gray-200/50 pt-2 text-xs text-gray-500">
                  {msg.turns.map((t, i) => (
                    <div key={i}>
                      <span className="font-medium">Turn {t.turn}:</span> {t.action}
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        ))}
        {sending && (
          <div className="flex justify-start">
            <div className="rounded-lg bg-gray-100 px-4 py-2.5 text-sm text-gray-500">
              Thinking...
            </div>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      {error && (
        <div className="border-t border-gray-200 bg-red-50 px-4 py-2 text-xs text-red-600">
          {error}
        </div>
      )}

      {/* Input area */}
      <div className="border-t border-gray-200 p-3">
        <div className="flex gap-2">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={sending || !sessionId}
            placeholder={sessionId ? 'Type a message...' : 'Connecting...'}
            className={cn(
              'flex-1 rounded-md border border-gray-300 px-3 py-2 text-sm',
              'focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500',
              'disabled:opacity-50',
            )}
          />
          <button
            onClick={sendMessage}
            disabled={sending || !input.trim() || !sessionId}
            className={cn(
              'rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white',
              'hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2',
              'disabled:opacity-50 disabled:cursor-not-allowed',
            )}
          >
            Send
          </button>
        </div>
      </div>
    </div>
  );
}
