/** 骨架屏加载占位组件 */
import { cn } from '../lib/cn';

interface SkeletonProps {
  className?: string;
  /** 行数（仅用于 text 模式） */
  lines?: number;
  /** 高度快捷设置 */
  height?: string;
  width?: string;
  /** 圆形 */
  circle?: boolean;
  /** 文本骨架（多条线） */
  text?: boolean;
}

export function Skeleton({ className, lines = 3, height, width, circle, text }: SkeletonProps) {
  if (circle) {
    return (
      <div
        className={cn('skeleton rounded-full shrink-0', className)}
        style={{ height: height ?? '40px', width: width ?? '40px' }}
      />
    );
  }

  if (text) {
    return (
      <div className={cn('space-y-2.5', className)}>
        {Array.from({ length: lines }).map((_, i) => (
          <div
            key={i}
            className="skeleton"
            style={{
              height: height ?? '14px',
              width: i === lines - 1 ? '60%' : '100%',
            }}
          />
        ))}
      </div>
    );
  }

  return (
    <div
      className={cn('skeleton', className)}
      style={{ height, width }}
    />
  );
}

/** 骨架卡片（封装了头部+内容区骨架） */
export function SkeletonCard({ className }: { className?: string }) {
  return (
    <div className={cn('glass-card rounded-md-xl p-5 space-y-4', className)}>
      <div className="flex items-center gap-3">
        <Skeleton circle height="36px" width="36px" />
        <div className="flex-1 space-y-2">
          <Skeleton height="14px" width="40%" />
          <Skeleton height="10px" width="60%" />
        </div>
      </div>
      <Skeleton text lines={2} />
    </div>
  );
}
