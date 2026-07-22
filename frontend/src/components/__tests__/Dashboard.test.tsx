import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Dashboard } from '../Dashboard';

// Mock the react-grid-layout and react-resizable
vi.mock('react-grid-layout', () => ({
  Responsive: ({ children }: { children: React.ReactNode }) => <div data-testid="responsive-grid">{children}</div>,
  useContainerWidth: () => ({ width: 1200, containerRef: { current: null }, mounted: true }),
}));

vi.mock('react-grid-layout/css/styles.css', () => ({}));
vi.mock('react-resizable/css/styles.css', () => ({}));

// Mock widget components
vi.mock('../widgets/HealthSummaryWidget', () => ({
  HealthSummaryWidget: () => <div data-testid="health-summary">HealthSummary</div>,
}));
vi.mock('../widgets/ModuleStatusWidget', () => ({
  ModuleStatusWidget: () => <div data-testid="module-status">ModuleStatus</div>,
}));
vi.mock('../widgets/QuickActionsWidget', () => ({
  QuickActionsWidget: () => <div data-testid="quick-actions">QuickActions</div>,
}));
vi.mock('../widgets/RecentAlertsWidget', () => ({
  RecentAlertsWidget: () => <div data-testid="recent-alerts">RecentAlerts</div>,
}));
vi.mock('../widgets/ResourceUsageWidget', () => ({
  ResourceUsageWidget: () => <div data-testid="resource-usage">ResourceUsage</div>,
}));

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => { store[key] = value; },
    removeItem: (key: string) => { delete store[key]; },
    clear: () => { store = {}; },
  };
})();
Object.defineProperty(window, 'localStorage', { value: localStorageMock });

beforeEach(() => {
  localStorageMock.clear();
});

describe('Dashboard', () => {
  it('shows loading skeleton initially', () => {
    render(<Dashboard />);
    expect(screen.getByText('OpsPilot')).toBeInTheDocument();
  });

  it('renders dashboard after loading', async () => {
    render(<Dashboard />);
    await waitFor(() => {
      expect(screen.getByText('OpsPilot')).toBeInTheDocument();
    }, { timeout: 1000 });
  });

  it('shows configure button', async () => {
    render(<Dashboard />);
    await waitFor(() => {
      expect(screen.getAllByText('dashboard.configure').length).toBeGreaterThan(0);
    }, { timeout: 1000 });
  });

  it('shows reset button', async () => {
    render(<Dashboard />);
    await waitFor(() => {
      expect(screen.getByText('dashboard.reset')).toBeInTheDocument();
    }, { timeout: 1000 });
  });

  it('shows subtitle', async () => {
    render(<Dashboard />);
    await waitFor(() => {
      expect(screen.getByText('dashboard.subtitle')).toBeInTheDocument();
    }, { timeout: 1000 });
  });
});
