import { useI18n } from '../../i18n';

export function QuickActionsWidget() {
  const { t } = useI18n();

  const actions = [
    { icon: '🖥️', label: t('widget.action.hosts'), tab: 'hosts' },
    { icon: '🛡️', label: t('widget.action.security'), tab: 'security' },
    { icon: '📊', label: t('widget.action.monitor'), tab: 'monitor' },
    { icon: '⏰', label: t('widget.action.scheduler'), tab: 'scheduler' },
    { icon: '📚', label: t('widget.action.knowledge'), tab: 'knowledge' },
    { icon: '💬', label: t('widget.action.chat'), tab: 'chat' },
  ];

  return (
    <div className="grid grid-cols-2 gap-2 h-full content-center">
      {actions.map((item) => (
        <a
          key={item.tab}
          href={`/${item.tab}`}
          className="flex flex-col items-center gap-1.5 glass-card rounded-md-lg px-2 py-3 text-md-on-surface-variant hover:text-md-primary hover:border-primary/40 transition-all duration-200 group"
        >
          <span className="text-xl group-hover:scale-110 transition-transform duration-200">{item.icon}</span>
          <span className="text-label-small text-center">{item.label}</span>
        </a>
      ))}
    </div>
  );
}
