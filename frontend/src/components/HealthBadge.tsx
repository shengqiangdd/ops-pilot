import type { HealthStatus } from '../api/types';
import { cn } from '../lib/cn';
import { getHealthLabel, getHealthColor, getHealthReason } from '../lib/health';

interface HealthBadgeProps {
  status: HealthStatus | string;
  className?: string;
}

export function HealthBadge({ status, className }: HealthBadgeProps) {
  const label = getHealthLabel(status);
  const reason = getHealthReason(status);
  return (
    <span
      className={cn('inline-flex items-center gap-1.5 text-body-medium', className)}
      title={reason ? `${label}: ${reason}` : label}
    >
      <span className={cn('h-2.5 w-2.5 rounded-full', getHealthColor(status))} />
      <span>{label}</span>
    </span>
  );
}
