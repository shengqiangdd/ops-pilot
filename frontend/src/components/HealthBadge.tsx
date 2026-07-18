import type { HealthStatus } from '../api/types';
import { cn } from '../lib/cn';

interface HealthBadgeProps {
  status: HealthStatus;
  className?: string;
}

function statusLabel(status: HealthStatus): string {
  if ('Healthy' in status) return 'Healthy';
  if ('Degraded' in status) return `Degraded: ${status.Degraded.reason}`;
  if ('Unhealthy' in status) return `Unhealthy: ${status.Unhealthy.reason}`;
  return 'Unknown';
}

function statusColor(status: HealthStatus): string {
  if ('Healthy' in status) return 'bg-green-500';
  if ('Degraded' in status) return 'bg-yellow-500';
  if ('Unhealthy' in status) return 'bg-red-500';
  return 'bg-gray-400';
}

export function HealthBadge({ status, className }: HealthBadgeProps) {
  return (
    <span
      className={cn('inline-flex items-center gap-1.5 text-sm', className)}
      title={statusLabel(status)}
    >
      <span className={cn('h-2.5 w-2.5 rounded-full', statusColor(status))} />
      <span>{'Healthy' in status ? 'Healthy' : 'Degraded' in status ? 'Degraded' : 'Unhealthy'}</span>
    </span>
  );
}
