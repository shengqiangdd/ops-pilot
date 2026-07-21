import { useCallback, useEffect, useRef, useState } from 'react';
import Markdown from 'react-markdown';
import { api } from '../api/client';
import type { AgentResponse } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  turns?: AgentResponse['turns'];
}

const QUICK_ACTIONS = [
  { id: 'diagnose', icon: '🔍', labelKey: 'chat.quick.diagnose', promptKey: 'chat.prompt.diagnose' },
  { id: 'knowledge', icon: '📚', labelKey: 'chat.quick.knowledge', promptKey: 'chat.prompt.knowledge' },
  { id: 'metrics', icon: '📊', labelKey: 'chat.quick.metrics', promptKey: 'chat.prompt.metrics' },
  { id: 'hosts', icon: '🖥️', labelKey: 'chat.quick.hosts', promptKey: 'chat.prompt.hosts' },
  { id: 'alerts', icon: '🔔', labelKey: 'chat.quick.alerts', promptKey: 'chat.prompt.alerts' },
  { id: 'runbook', icon: '📋', labelKey: 'chat.quick.runbook', promptKey: 'chat.prompt.runbook' },
];

export function AdvisorChat() {
  const token = useAuthStore((s) => s.token);
  const { t } = useI18n();
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [sending, setSending] = useState(false);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => { bottomRef.current?.scrollIntoView({ behavior: 'smooth' }); }, [messages]);

  useEffect(() => {
    if (!token) return;
    let cancelled = false;
    api.createAgentSession(token)
      .then((s) => { if (!cancelled) setSessionId(s.session_id); })
      .catch((e) => { if (!cancelled) setError(e instanceof Error ? e.message : 'Failed to create session'); });
    return () => { cancelled = true; };
  }, [token]);

  const sendMessage = useCallback(async (text?: string) => {
    const msgText = (text || input).trim();
    if (!msgText || !sessionId || !token || sending) return;
    setInput('');
    setError(null);
    const userMsg: ChatMessage = { id: `user-${Date.now()}`, role: 'user', content: msgText };
    setMessages((prev) => [...prev, userMsg]);
    setSending(true);
    try {
      const resp: AgentResponse = await api.sendAgentMessage(token, sessionId, msgText);
      const assistantMsg: ChatMessage = { id: `assistant-${Date.now()}`, role: 'assistant', content: resp.content, turns: resp.turns };
      setMessages((prev) => [...prev, assistantMsg]);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to send message');
    } finally { setSending(false); }
  }, [input, sessionId, token, sending]);

  const handleQuickAction = (actionId: string) => {
    const action = QUICK_ACTIONS.find(a => a.id === actionId);
    if (action) {
      setInput(t(action.promptKey));
    }
  };

  const handleSubmit = () => { sendMessage(); };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleSubmit(); }
  };

  if (!token) {
    return (
      <div className="rounded-md-xl bg-md-surface-container-low p-8 text-center text-md-on-surface-variant shadow-md-1">
        {t('chat.login_required')}
      </div>
    );
  }

  return (
    <div className="flex flex-col rounded-md-xl bg-md-surface-container-low shadow-md-1" style={{ height: 'calc(100vh - 12rem)' }}>
      {/* Quick Actions */}
      <div className="border-b border-md-outline-variant p-3">
        <div className="flex flex-wrap gap-2">
          {QUICK_ACTIONS.map((action) => (
            <button
              key={action.id}
              onClick={() => handleQuickAction(action.id)}
              className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md-full glass-card hover:bg-md-surface-container-high transition-colors text-md-on-surface-variant hover:text-md-on-surface"
            >
              <span>{action.icon}</span>
              <span>{t(action.labelKey)}</span>
            </button>
          ))}
        </div>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {messages.length === 0 && (
          <div className="flex h-full items-center justify-center text-body-medium text-md-outline">
            {t('chat.start')}
          </div>
        )}
        {messages.map((msg) => (
          <div key={msg.id} className={cn('flex', msg.role === 'user' ? 'justify-end' : 'justify-start')}>
            <div className={cn(
              'max-w-[80%] rounded-md-lg px-4 py-2.5 text-body-medium leading-relaxed',
              msg.role === 'user'
                ? 'bg-md-primary text-md-on-primary'
                : 'bg-md-surface-container-high text-md-on-surface',
            )}>
              {msg.role === 'assistant' ? (
                <div className="prose prose-sm dark:prose-invert max-w-none">
                  <Markdown>{msg.content}</Markdown>
                </div>
              ) : (
                <div className="whitespace-pre-wrap">{msg.content}</div>
              )}
              {msg.turns && msg.turns.length > 0 && (
                <div className="mt-2 border-t border-md-outline-variant pt-2 text-label-medium text-md-on-surface-variant">
                  {msg.turns.map((turn, i) => (
                    <div key={i}>
                      <span className="font-medium">{t('chat.step')} {turn.turn}:</span> {turn.action}
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        ))}
        {sending && (
          <div className="flex justify-start">
            <div className="rounded-md-lg bg-md-surface-container-high px-4 py-2.5 text-body-medium text-md-on-surface-variant">
              {t('chat.thinking')}
            </div>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      {/* Error */}
      {error && (
        <div className="border-t border-md-outline-variant bg-md-error-container px-4 py-2 text-label-medium text-md-on-error-container">
          {error}
        </div>
      )}

      {/* Input */}
      <div className="border-t border-md-outline-variant p-3">
        <div className="flex gap-2">
          <input type="text" value={input} onChange={(e) => setInput(e.target.value)} onKeyDown={handleKeyDown}
            disabled={sending || !sessionId}
            placeholder={sessionId ? t('chat.placeholder') : t('chat.connecting')}
            className="flex-1 bg-md-surface-container-highest rounded-md-sm px-4 py-2.5 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface disabled:opacity-50" />
          <button onClick={handleSubmit} disabled={sending || !input.trim() || !sessionId}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
            {t('chat.send')}
          </button>
        </div>
        <p className="text-label-small text-md-on-surface-variant mt-2">
          {t('chat.hint')}
        </p>
      </div>
    </div>
  );
}
