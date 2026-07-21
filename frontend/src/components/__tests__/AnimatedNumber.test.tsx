import { render, screen, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeAll, afterAll } from 'vitest';
import { AnimatedNumber } from '../AnimatedNumber';

// Stable mock for requestAnimationFrame / cancelAnimationFrame
let rafId = 0;
const rafCallbacks = new Map<number, FrameRequestCallback>();

beforeAll(() => {
  vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => {
    const id = ++rafId;
    rafCallbacks.set(id, cb);
    return id;
  });
  vi.stubGlobal('cancelAnimationFrame', (id: number) => {
    rafCallbacks.delete(id);
  });
});

afterAll(() => {
  vi.restoreAllMocks();
});

function flushRaf(ms: number) {
  act(() => {
    const now = performance.now() + ms;
    for (const [id, cb] of rafCallbacks) {
      rafCallbacks.delete(id);
      cb(now);
    }
  });
}

describe('AnimatedNumber', () => {
  it('renders initial value (starts at 0)', () => {
    render(<AnimatedNumber value={100} />);
    expect(screen.getByText('0')).toBeTruthy();
  });

  it('animates to target value after flushing frames', () => {
    render(<AnimatedNumber value={50} duration={100} />);
    // Flush enough frames for the animation to complete
    for (let i = 0; i < 20; i++) {
      flushRaf(100);
    }
    expect(screen.getByText('50')).toBeTruthy();
  });

  it('renders suffix', () => {
    render(<AnimatedNumber value={10} duration={10} suffix="%" />);
    for (let i = 0; i < 20; i++) {
      flushRaf(100);
    }
    expect(screen.getByText('10%')).toBeTruthy();
  });

  it('renders prefix', () => {
    render(<AnimatedNumber value={10} duration={10} prefix="$" />);
    for (let i = 0; i < 20; i++) {
      flushRaf(100);
    }
    expect(screen.getByText('$10')).toBeTruthy();
  });

  it('renders prefix and suffix together', () => {
    render(<AnimatedNumber value={100} duration={10} prefix="$" suffix="k" />);
    for (let i = 0; i < 20; i++) {
      flushRaf(100);
    }
    expect(screen.getByText('$100k')).toBeTruthy();
  });
});
