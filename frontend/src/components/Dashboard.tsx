import { useCallback, useEffect, useMemo, useState } from 'react';
import { Responsive, useContainerWidth } from 'react-grid-layout';
import 'react-grid-layout/css/styles.css';
import 'react-resizable/css/styles.css';
import { cn } from '../lib/cn';
import { Skeleton, SkeletonCard } from './Skeleton';
import { useI18n } from '../i18n';
import { AVAILABLE_WIDGETS, LAYOUT_KEY, ENABLED_WIDGETS_KEY } from './widgets/WidgetType';
import { HealthSummaryWidget } from './widgets/HealthSummaryWidget';
import { ModuleStatusWidget } from './widgets/ModuleStatusWidget';
import { QuickActionsWidget } from './widgets/QuickActionsWidget';
import { RecentAlertsWidget } from './widgets/RecentAlertsWidget';
import { ResourceUsageWidget } from './widgets/ResourceUsageWidget';
import type { LayoutItem } from 'react-grid-layout';

const WIDGET_COMPONENTS: Record<string, React.FC> = {
  'health-summary': HealthSummaryWidget,
  'module-status': ModuleStatusWidget,
  'quick-actions': QuickActionsWidget,
  'recent-alerts': RecentAlertsWidget,
  'resource-usage': ResourceUsageWidget,
};

function loadLayout(): LayoutItem[] {
  try {
    const saved = localStorage.getItem(LAYOUT_KEY);
    if (saved) return JSON.parse(saved);
  } catch { /* ignore */ }
  return AVAILABLE_WIDGETS.map((w, i) => ({
    i: w.id,
    x: (i % 2) * 6,
    y: Math.floor(i / 2) * 2,
    w: w.defaultW,
    h: w.defaultH,
    minW: w.minW,
    minH: w.minH,
  }));
}

function loadEnabledWidgets(): string[] {
  try {
    const saved = localStorage.getItem(ENABLED_WIDGETS_KEY);
    if (saved) return JSON.parse(saved);
  } catch { /* ignore */ }
  return AVAILABLE_WIDGETS.map(w => w.id);
}

/* ── Widget Card Wrapper ── */
function WidgetCard({ id, title, onRemove, children }: {
  id: string; title: string; onRemove: (id: string) => void; children: React.ReactNode;
}) {
  return (
    <div className="glass-card rounded-md-xl p-4 h-full flex flex-col animate-scale-in overflow-hidden">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-title-small font-semibold text-md-on-surface truncate">{title}</h3>
        <button
          onClick={() => onRemove(id)}
          className="w-6 h-6 rounded-md-full flex items-center justify-center text-md-on-surface-variant hover:bg-md-surface-container-high hover:text-md-error transition-colors shrink-0"
          title="移除小部件"
        >
          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
      <div className="flex-1 min-h-0 overflow-auto">{children}</div>
    </div>
  );
}

/* ── Widget Picker Panel ── */
function WidgetPickerPanel({
  enabledWidgets,
  onToggle,
  onClose,
}: {
  enabledWidgets: string[];
  onToggle: (id: string) => void;
  onClose: () => void;
}) {
  const { t } = useI18n();
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" onClick={onClose}>
      <div className="glass-card rounded-md-2xl p-6 w-full max-w-md shadow-md-3 animate-scale-in" onClick={e => e.stopPropagation()}>
        <div className="flex items-center justify-between mb-5">
          <h2 className="text-title-large font-semibold text-md-on-surface">{t('dashboard.configure')}</h2>
          <button onClick={onClose} className="w-8 h-8 rounded-md-full flex items-center justify-center hover:bg-md-surface-container-high transition-colors">
            <svg className="w-5 h-5 text-md-on-surface-variant" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div className="space-y-2">
          {AVAILABLE_WIDGETS.map(w => {
            const enabled = enabledWidgets.includes(w.id);
            return (
              <label
                key={w.id}
                className={cn(
                  'flex items-center gap-3 px-4 py-3 rounded-md-lg cursor-pointer transition-colors',
                  enabled ? 'bg-md-primary-container/30' : 'hover:bg-md-surface-container-high',
                )}
              >
                <span className="text-xl">{w.icon}</span>
                <span className="flex-1 text-body-medium text-md-on-surface">{t(w.titleKey)}</span>
                <div className={cn(
                  'relative w-10 h-6 rounded-full transition-colors',
                  enabled ? 'bg-md-primary' : 'bg-md-surface-container-highest',
                )}>
                  <div className={cn(
                    'absolute top-1 w-4 h-4 rounded-full bg-white shadow transition-transform',
                    enabled ? 'translate-x-5' : 'translate-x-1',
                  )} />
                </div>
                <input
                  type="checkbox"
                  checked={enabled}
                  onChange={() => onToggle(w.id)}
                  className="sr-only"
                />
              </label>
            );
          })}
        </div>
      </div>
    </div>
  );
}

/* ── 主仪表盘 ── */
export function Dashboard() {
  const { t } = useI18n();
  const [layouts, setLayouts] = useState<LayoutItem[]>(loadLayout);
  const [enabledWidgets, setEnabledWidgets] = useState<string[]>(loadEnabledWidgets);
  const [showPicker, setShowPicker] = useState(false);
  const [loading, setLoading] = useState(true);
  const { width, containerRef, mounted } = useContainerWidth();

  // Simulate initial loading
  useEffect(() => {
    const timer = setTimeout(() => setLoading(false), 600);
    return () => clearTimeout(timer);
  }, []);

  const handleLayoutChange = useCallback((_currentLayout: readonly LayoutItem[], allLayouts: Record<string, readonly LayoutItem[]>) => {
    if (allLayouts.lg) {
      const newLayout = [...allLayouts.lg];
      setLayouts(newLayout);
      try { localStorage.setItem(LAYOUT_KEY, JSON.stringify(newLayout)); } catch { /* ignore */ }
    }
  }, []);

  const toggleWidget = useCallback((id: string) => {
    setEnabledWidgets(prev => {
      const next = prev.includes(id) ? prev.filter(w => w !== id) : [...prev, id];
      try { localStorage.setItem(ENABLED_WIDGETS_KEY, JSON.stringify(next)); } catch { /* ignore */ }
      return next;
    });
  }, []);

  const removeWidget = useCallback((id: string) => {
    toggleWidget(id);
  }, [toggleWidget]);

  const resetLayout = useCallback(() => {
    const defaultLayout = AVAILABLE_WIDGETS.map((w, i) => ({
      i: w.id,
      x: (i % 2) * 6,
      y: Math.floor(i / 2) * 2,
      w: w.defaultW,
      h: w.defaultH,
      minW: w.minW,
      minH: w.minH,
    }));
    setLayouts(defaultLayout);
    setEnabledWidgets(AVAILABLE_WIDGETS.map(w => w.id));
    try {
      localStorage.setItem(LAYOUT_KEY, JSON.stringify(defaultLayout));
      localStorage.setItem(ENABLED_WIDGETS_KEY, JSON.stringify(AVAILABLE_WIDGETS.map(w => w.id)));
    } catch { /* ignore */ }
  }, []);

  const filteredLayouts = useMemo(
    () => layouts.filter(l => enabledWidgets.includes(l.i)),
    [layouts, enabledWidgets],
  );

  if (loading) {
    return (
      <div className="space-y-6 animate-slide-up">
        <div className="flex items-center justify-between">
          <div>
            <Skeleton height="28px" width="180px" />
            <Skeleton height="14px" width="240px" className="mt-2" />
          </div>
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          {Array.from({ length: 4 }).map((_, i) => <SkeletonCard key={i} />)}
        </div>
        <div className="glass-card rounded-md-xl p-5 space-y-3">
          {Array.from({ length: 6 }).map((_, i) => (
            <div key={i} className="flex items-center gap-3">
              <Skeleton circle height="10px" width="10px" />
              <Skeleton height="14px" width="30%" />
              <Skeleton height="12px" width="50px" className="ml-auto" />
            </div>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6 animate-slide-up">
      {/* 头部 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-headline-medium gradient-text">OpsPilot</h1>
          <p className="text-body-medium text-md-on-surface-variant mt-1">
            {t('dashboard.subtitle')}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={resetLayout}
            className="glass-card rounded-md-lg px-4 py-2.5 text-body-medium font-medium text-md-on-surface-variant hover:text-md-on-surface transition-all flex items-center gap-2"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            {t('dashboard.reset')}
          </button>
          <button
            onClick={() => setShowPicker(true)}
            className="glass-card rounded-md-lg px-4 py-2.5 text-body-medium font-medium text-md-primary hover:shadow-md-2 active:scale-[0.97] transition-all flex items-center gap-2"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            {t('dashboard.configure')}
          </button>
        </div>
      </div>

      {/* 网格布局 */}
      <div ref={containerRef}>
        {mounted && width > 0 && (
          <Responsive
            className="layout"
            layouts={{ lg: filteredLayouts }}
            breakpoints={{ lg: 1200, md: 996, sm: 768, xs: 480, xxs: 0 }}
            cols={{ lg: 12, md: 8, sm: 6, xs: 4, xxs: 2 }}
            rowHeight={80}
            width={width}
            dragConfig={{ enabled: true, bounded: false, handle: '.drag-handle', threshold: 3 }}
            resizeConfig={{ enabled: true, handles: ['se'] }}
            onLayoutChange={handleLayoutChange}
          >
            {filteredLayouts.map(item => {
              const widgetDef = AVAILABLE_WIDGETS.find(w => w.id === item.i);
              if (!widgetDef) return null;
              const WidgetComponent = WIDGET_COMPONENTS[item.i];
              if (!WidgetComponent) return null;
              return (
                <div key={item.i}>
                  <WidgetCard
                    id={item.i}
                    title={t(widgetDef.titleKey)}
                    onRemove={removeWidget}
                  >
                    <div className="drag-handle cursor-grab active:cursor-grabbing absolute top-0 left-0 right-0 h-6 z-10" />
                    <WidgetComponent />
                  </WidgetCard>
                </div>
              );
            })}
          </Responsive>
        )}
      </div>

      {/* Widget 选择面板 */}
      {showPicker && (
        <WidgetPickerPanel
          enabledWidgets={enabledWidgets}
          onToggle={toggleWidget}
          onClose={() => setShowPicker(false)}
        />
      )}
    </div>
  );
}
