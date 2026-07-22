import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi } from 'vitest';
import { MetricGrid } from '../MetricGrid';
import type { MetricSeries } from '../../lib/metrics';

vi.mock('../MetricChart', () => ({
  MetricChart: ({ title }: { title: string }) => <div data-testid="metric-chart">{title}</div>,
}));

const mockMetrics: MetricSeries[] = [
  {
    name: 'CPU %',
    data: [
      { timestamp: '2026-01-01T00:00:00Z', value: 50 },
      { timestamp: '2026-01-01T00:01:00Z', value: 60 },
    ],
    color: '#6750A4',
  },
  {
    name: 'Memory %',
    data: [
      { timestamp: '2026-01-01T00:00:00Z', value: 70 },
      { timestamp: '2026-01-01T00:01:00Z', value: 75 },
    ],
    color: '#7D5260',
  },
];

describe('MetricGrid', () => {
  it('renders all metric charts', () => {
    render(<MetricGrid metrics={mockMetrics} />);
    expect(screen.getByText('CPU %')).toBeInTheDocument();
    expect(screen.getByText('Memory %')).toBeInTheDocument();
  });

  it('renders title when provided', () => {
    render(<MetricGrid metrics={mockMetrics} title="System Metrics" />);
    expect(screen.getByText('System Metrics')).toBeInTheDocument();
  });

  it('renders without title when not provided', () => {
    const { container } = render(<MetricGrid metrics={mockMetrics} />);
    expect(container.querySelector('.text-title-medium')).not.toBeInTheDocument();
  });

  it('opens fullscreen on chart click', async () => {
    const user = userEvent.setup();
    render(<MetricGrid metrics={mockMetrics} />);

    await user.click(screen.getByText('CPU %'));

    expect(screen.getByText('CPU % — Detailed View')).toBeInTheDocument();
  });

  it('closes fullscreen on backdrop click', async () => {
    const user = userEvent.setup();
    render(<MetricGrid metrics={mockMetrics} />);

    await user.click(screen.getByText('CPU %'));
    expect(screen.getByText('CPU % — Detailed View')).toBeInTheDocument();

    // Click the backdrop
    const backdrop = screen.getByText('CPU % — Detailed View').closest('.fixed');
    const backdropChild = backdrop?.querySelector('.glass-card')?.parentElement;
    if (backdropChild) {
      await user.click(backdropChild);
    } else {
      await user.click(backdrop!);
    }

    // Wait for fullscreen to close
    expect(screen.queryByText('CPU % — Detailed View')).not.toBeInTheDocument();
  });

  it('shows stats in fullscreen modal', async () => {
    const user = userEvent.setup();
    render(<MetricGrid metrics={mockMetrics} />);

    await user.click(screen.getByText('Memory %'));

    expect(screen.getByText('Current')).toBeInTheDocument();
    expect(screen.getByText('Min')).toBeInTheDocument();
    expect(screen.getByText('Average')).toBeInTheDocument();
    expect(screen.getByText('Max')).toBeInTheDocument();
  });

  it('renders with thresholds', () => {
    render(
      <MetricGrid metrics={mockMetrics} thresholds={{ 'CPU %': 80 }} />
    );
    expect(screen.getByText('CPU %')).toBeInTheDocument();
  });
});
