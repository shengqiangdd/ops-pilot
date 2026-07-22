import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MetricChart } from '../MetricChart';
import type { MetricDataPoint } from '../../lib/metrics';

vi.mock('recharts', () => ({
  ResponsiveContainer: ({ children }: any) => <div data-testid="responsive-container">{children}</div>,
  AreaChart: ({ children }: any) => <div data-testid="area-chart">{children}</div>,
  Area: () => <div data-testid="area" />,
  XAxis: () => <div data-testid="x-axis" />,
  YAxis: () => <div data-testid="y-axis" />,
  CartesianGrid: () => <div data-testid="cartesian-grid" />,
  Tooltip: (props: any) => {
    const CustomTooltip = props.content;
    if (CustomTooltip) {
      return <div data-testid="tooltip"><CustomTooltip active={false} payload={[]} /></div>;
    }
    return <div data-testid="tooltip" />;
  },
  ReferenceLine: () => <div data-testid="reference-line" />,
  Brush: () => <div data-testid="brush" />,
}));

const mockData: MetricDataPoint[] = [
  { timestamp: '2026-01-01T00:00:00Z', value: 50 },
  { timestamp: '2026-01-01T00:01:00Z', value: 60 },
  { timestamp: '2026-01-01T00:02:00Z', value: 55 },
];

describe('MetricChart', () => {
  it('renders title', () => {
    render(<MetricChart data={mockData} title="CPU Usage" color="#6750A4" />);
    expect(screen.getByText('CPU Usage')).toBeInTheDocument();
  });

  it('renders stats', () => {
    render(<MetricChart data={mockData} title="CPU Usage" color="#6750A4" />);
    expect(screen.getByText('Min:')).toBeInTheDocument();
    expect(screen.getByText('Avg:')).toBeInTheDocument();
    expect(screen.getByText('Max:')).toBeInTheDocument();
  });

  it('renders chart with responsive container', () => {
    const { container } = render(<MetricChart data={mockData} title="CPU" color="#6750A4" />);
    expect(container.querySelector('[data-testid="responsive-container"]')).toBeInTheDocument();
  });

  it('renders with unit', () => {
    render(<MetricChart data={mockData} title="Memory" color="#7D5260" unit="MB" />);
    expect(screen.getByText('Min:')).toBeInTheDocument();
  });

  it('renders threshold line when provided', () => {
    const { container } = render(
      <MetricChart data={mockData} title="CPU" color="#6750A4" threshold={80} thresholdLabel="Warning" />
    );
    // Threshold should trigger ReferenceLine rendering
    expect(container.querySelector('[data-testid="reference-line"]')?.textContent).toBeDefined();
  });

  it('renders with Brush when showBrush is true', () => {
    const { container } = render(
      <MetricChart data={mockData} title="CPU" color="#6750A4" showBrush />
    );
    expect(container.querySelector('[data-testid="brush"]')).toBeInTheDocument();
  });

  it('renders empty data gracefully', () => {
    const { container } = render(<MetricChart data={[]} title="Empty" color="#6750A4" />);
    expect(screen.getByText('Empty')).toBeInTheDocument();
  });
});
