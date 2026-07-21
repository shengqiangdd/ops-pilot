import type { Shortcut } from '../hooks/useKeyboardShortcuts';
import { useI18n } from '../i18n';

interface Props {
  shortcuts: Shortcut[];
  open: boolean;
  onClose: () => void;
}

export function ShortcutHelp({ shortcuts, open, onClose }: Props) {
  const { t } = useI18n();

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="glass-card rounded-md-xl p-6 w-full max-w-lg mx-4 shadow-2xl animate-scale-in"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-title-large text-md-on-surface font-semibold">
            {t('shortcut.help_title')}
          </h2>
          <button
            onClick={onClose}
            className="text-md-on-surface-variant hover:text-md-on-surface transition-colors p-1"
            aria-label={t('nav.back')}
          >
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="space-y-2 max-h-96 overflow-y-auto">
          {shortcuts.map((s, i) => (
            <div
              key={i}
              className="flex items-center justify-between py-2 px-3 rounded-md-lg hover:bg-md-surface-container-high transition-colors"
            >
              <span className="text-body-medium text-md-on-surface-variant">
                {s.description}
              </span>
              <kbd className="inline-flex items-center gap-1 px-2 py-1 text-xs font-mono rounded-md bg-md-surface-container-high text-md-on-surface-variant border border-md-outline-variant">
                {s.ctrl && <span>⌘</span>}
                {s.shift && <span>⇧</span>}
                {s.alt && <span>⌥</span>}
                <span className="uppercase">{s.key}</span>
              </kbd>
            </div>
          ))}
        </div>

        <p className="mt-4 text-body-small text-md-on-surface-variant text-center">
          {t('shortcut.hint')}
        </p>
      </div>
    </div>
  );
}
