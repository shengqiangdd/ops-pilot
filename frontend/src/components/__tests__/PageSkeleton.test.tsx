import { render } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { ListPageSkeleton, ChartPageSkeleton, FormPageSkeleton, DetailPageSkeleton } from '../PageSkeleton';

describe('PageSkeleton', () => {
  it('renders ListPageSkeleton with rows', () => {
    const { container } = render(<ListPageSkeleton />);
    // Should have 8 skeleton rows
    const bars = container.querySelectorAll('.animate-pulse');
    expect(bars.length).toBeGreaterThan(0);
  });

  it('renders ChartPageSkeleton with cards and chart area', () => {
    const { container } = render(<ChartPageSkeleton />);
    const bars = container.querySelectorAll('.animate-pulse');
    expect(bars.length).toBeGreaterThan(0);
  });

  it('renders FormPageSkeleton with input placeholders', () => {
    const { container } = render(<FormPageSkeleton />);
    const bars = container.querySelectorAll('.animate-pulse');
    expect(bars.length).toBeGreaterThan(0);
  });

  it('renders DetailPageSkeleton with card layout', () => {
    const { container } = render(<DetailPageSkeleton />);
    const bars = container.querySelectorAll('.animate-pulse');
    expect(bars.length).toBeGreaterThan(0);
  });

  it('all skeletons have animate-pulse class', () => {
    const { container: c1 } = render(<ListPageSkeleton />);
    const { container: c2 } = render(<ChartPageSkeleton />);
    const { container: c3 } = render(<FormPageSkeleton />);
    const { container: c4 } = render(<DetailPageSkeleton />);
    [c1, c2, c3, c4].forEach(c => {
      expect(c.querySelectorAll('.animate-pulse').length).toBeGreaterThan(0);
    });
  });
});
