import { describe, it, expect } from 'vitest';
import { cn } from '../cn';

describe('cn', () => {
  it('merges class names', () => {
    expect(cn('px-4', 'py-2')).toBe('px-4 py-2');
  });

  it('handles conditional classes', () => {
    expect(cn('base', false && 'hidden', true && 'visible')).toBe('base visible');
  });

  it('handles clsx array arguments', () => {
    expect(cn(['foo', 'bar'], 'baz')).toBe('foo bar baz');
  });

  it('resolves tailwind conflicts', () => {
    // tailwind-merge should resolve the conflict: px-4 overrides px-2
    const result = cn('px-2', 'px-4');
    expect(result).toBe('px-4');
  });

  it('handles undefined and null values', () => {
    expect(cn('a', undefined, null, 'b')).toBe('a b');
  });

  it('merges multiple utility classes', () => {
    expect(cn('text-red-500', 'bg-blue-500', 'rounded-lg')).toBe('text-red-500 bg-blue-500 rounded-lg');
  });

  it('handles empty inputs', () => {
    expect(cn()).toBe('');
  });

  it('resolves conflicting padding', () => {
    const result = cn('p-4', 'p-2');
    expect(result).toBe('p-2');
  });

  it('resolves conflicting margin', () => {
    const result = cn('m-4', 'm-2');
    expect(result).toBe('m-2');
  });

  it('handles object syntax', () => {
    expect(cn({ foo: true, bar: false })).toBe('foo');
  });
});
