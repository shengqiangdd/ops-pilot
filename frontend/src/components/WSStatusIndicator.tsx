import { useWebSocketStatus } from '../hooks/useWebSocketStatus';
import { useI18n } from '../i18n';

/**
 * WebSocket 连接状态指示器
 * 显示在顶栏右侧，用绿/黄/红圆点表示连接状态
 */
export function WSStatusIndicator() {
  const status = useWebSocketStatus();
  const { t } = useI18n();

  const dotColor = {
    connected: 'bg-green-500',
    connecting: 'bg-amber-500',
    disconnected: 'bg-red-500',
  }[status];

  const label = {
    connected: t('terminal.status.connected'),
    connecting: t('terminal.status.connecting'),
    disconnected: t('terminal.status.disconnected'),
  }[status];

  return (
    <div
      className="flex items-center gap-1.5 px-2 py-1 rounded-md-md text-xs text-md-on-surface-variant"
      title={`${t('ws.status')}: ${label}`}
    >
      <span className={`h-2 w-2 rounded-full ${dotColor} ${status === 'connecting' ? 'animate-pulse' : ''}`} />
      <span className="hidden sm:inline">{label}</span>
    </div>
  );
}
