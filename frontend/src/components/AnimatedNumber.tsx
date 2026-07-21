import { useEffect, useRef, useState } from 'react';

interface Props {
  value: number;
  duration?: number;
  suffix?: string;
  prefix?: string;
  className?: string;
}

export function AnimatedNumber({ value, duration = 1200, suffix = '', prefix = '', className }: Props) {
  const [display, setDisplay] = useState(0);
  const frameRef = useRef<number>(0);
  const startRef = useRef<number>(0);
  const startValueRef = useRef(0);

  useEffect(() => {
    startValueRef.current = display;
    startRef.current = performance.now();

    const decimals = value % 1 !== 0 ? Math.max(String(value).split('.')[1]?.length || 0, 1) : 0;

    const animate = (now: number) => {
      const elapsed = now - startRef.current;
      const progress = Math.min(elapsed / duration, 1);
      const eased = 1 - Math.pow(1 - progress, 3);
      const current = startValueRef.current + (value - startValueRef.current) * eased;
      setDisplay(Number(current.toFixed(decimals)));
      if (progress < 1) {
        frameRef.current = requestAnimationFrame(animate);
      }
    };

    frameRef.current = requestAnimationFrame(animate);
    return () => cancelAnimationFrame(frameRef.current);
  }, [value, duration]);

  return (
    <span className={className}>
      {prefix}{display}{suffix}
    </span>
  );
}
