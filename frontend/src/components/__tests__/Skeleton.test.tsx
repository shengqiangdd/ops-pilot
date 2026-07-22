import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Skeleton, SkeletonCard } from '../Skeleton';

describe('Skeleton', () => {
  it('renders default skeleton', () => {
    const { container } = render(<Skeleton />);
    expect(container.querySelector('.skeleton')).toBeInTheDocument();
  });

  it('renders circle skeleton', () => {
    const { container } = render(<Skeleton circle />);
    const el = container.querySelector('.skeleton');
    expect(el).toBeInTheDocument();
    expect(el).toHaveClass('rounded-full');
  });

  it('renders text skeleton with multiple lines', () => {
    const { container } = render(<Skeleton text lines={4} />);
    expect(container.querySelectorAll('.skeleton')).toHaveLength(4);
  });

  it('renders text skeleton with default lines', () => {
    const { container } = render(<Skeleton text />);
    expect(container.querySelectorAll('.skeleton')).toHaveLength(3);
  });

  it('applies custom height and width', () => {
    const { container } = render(<Skeleton height="50px" width="100px" />);
    const el = container.querySelector('.skeleton') as HTMLElement;
    expect(el.style.height).toBe('50px');
    expect(el.style.width).toBe('100px');
  });

  it('applies custom className', () => {
    const { container } = render(<Skeleton className="custom-class" />);
    const el = container.querySelector('.skeleton');
    expect(el).toHaveClass('custom-class');
  });
});

describe('SkeletonCard', () => {
  it('renders skeleton card', () => {
    const { container } = render(<SkeletonCard />);
    expect(container.querySelector('.glass-card')).toBeInTheDocument();
  });
});
