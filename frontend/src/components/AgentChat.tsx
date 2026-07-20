import { useCallback, useEffect, useRef, useState } from 'react';
import Markdown from 'react-markdown';
import { api } from '../api/client';
import type { AgentResponse, NlQueryResponse, DiagnoseResponse } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';
import { useI18n } from '../i18n';

interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  turns?: AgentResponse['turns'];
  type?: 'normal' | 'nl-query' | 'diagnose';
}

const QUICK_ACTIONS = [
  { id: 'diagnose', icon: '🔍', labelKey: 'chat.quick.diagnose', prompt: '请帮我诊断主机问题：' },
  { id: 'knowledge', icon: '📚', labelKey: 'chat.quick.knowledge', prompt: '搜索知识库：' },
  { id: 'metrics', icon: '📊', labelKey: 'chat.quick.metrics', prompt: '查询主机指标：' },
  { id: 'hosts', icon: '🖥️', labelKey: 'chat.quick.hosts', prompt: '列出所有主机状态' },
  { id: 'alerts', icon: '🔔', labelKey: 'chat.quick.alerts', prompt: '查看最近告警' },
];

export function AgentChat() {
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

  const handleNlQuery = useCallback(async (query: string) => {
    if (!token) return;
    const userMsg: ChatMessage = { id: `user-${Date.now()}`, role: 'user', content: query, type: 'nl-query' };
    setMessages((prev) => [...prev, userMsg]);
    setSending(true);
    try {
      const resp: NlQueryResponse = await api.nlQuery(token, query);
      const content = `**查询**: ${resp.query}\n\n**意图**: ${resp.parsed_intent}\n\n**结果**: ${resp.summary}`;
      const assistantMsg: ChatMessage = { id: `assistant-${Date.now()}`, role: 'assistant', content, type: 'nl-query' };
      setMessages((prev) => [...prev, assistantMsg]);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Query failed');
    } finally { setSending(false); }
  }, [token]);

  const handleDiagnose = useCallback(async (description: string) => {
    if (!token) return;
    const userMsg: ChatMessage = { id: `user-${Date.now()}`, role: 'user', content: `诊断: ${description}`, type: 'diagnose' };
    setMessages((prev) => [...prev, userMsg]);
    setSending(true);
    try {
      const resp: DiagnoseResponse = await api.diagnose(token, { issue_description: description });
      let content = `## 诊断报告\n\n**问题**: ${resp.issue}\n\n**严重级别**: ${resp.severity}\n\n`;
      content += `### 可能原因\n`;
      resp.possible_causes.forEach((c, i) => { content += `${i + 1}. ${c}\n`; });
      content += `\n### 建议操作\n`;
      resp.recommended_actions.forEach((a, i) => { content += `${i + 1}. ${a}\n`; });
      const assistantMsg: ChatMessage = { id: `assistant-${Date.now()}`, role: 'assistant', content, type: 'diagnose' };
      setMessages((prev) => [...prev, assistantMsg]);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Diagnosis failed');
    } finally { setSending(false); }
  }, [token]);

  const handleQuickAction = (actionId: string) => {
    const action = QUICK_ACTIONS.find(a => a.id === actionId);
    if (action) {
      setInput(action.prompt);
    }
  };

  const handleSubmit = () => {
    const text = input.trim();
    if (!text) return;

    // Detect intent
    if (text.startsWith('诊断:') || text.startsWith('diagnose:')) {
      handleDiagnose(text.replace(/^(诊断|diagnose):/i, '').trim());
    } else if (text.startsWith('查询:') || text.startsWith('query:')) {
      handleNlQuery(text.replace(/^(查询|query):/i, '').trim());
    } else {
      sendMessage();
    }
  };

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
              msg.type === 'diagnose' && 'bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800',
              msg.type === 'nl-query' && 'bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800',
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
