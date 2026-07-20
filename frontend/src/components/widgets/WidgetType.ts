export interface WidgetDefinition {
  id: string;
  titleKey: string;
  icon: string;
  defaultW: number;
  defaultH: number;
  minW?: number;
  minH?: number;
}

export const AVAILABLE_WIDGETS: WidgetDefinition[] = [
  { id: 'health-summary', titleKey: 'widget.health_summary', icon: '❤️', defaultW: 4, defaultH: 1 },
  { id: 'module-status', titleKey: 'widget.module_status', icon: '🧩', defaultW: 2, defaultH: 2, minW: 2, minH: 2 },
  { id: 'quick-actions', titleKey: 'widget.quick_actions', icon: '⚡', defaultW: 1, defaultH: 2 },
  { id: 'recent-alerts', titleKey: 'widget.recent_alerts', icon: '🔔', defaultW: 2, defaultH: 2, minW: 2, minH: 1 },
  { id: 'resource-usage', titleKey: 'widget.resource_usage', icon: '📊', defaultW: 2, defaultH: 1 },
];

export const LAYOUT_KEY = 'opspilot-dashboard-layout';
export const ENABLED_WIDGETS_KEY = 'opspilot-enabled-widgets';
