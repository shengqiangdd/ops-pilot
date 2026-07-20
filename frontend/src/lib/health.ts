/**
 * HealthStatus 兼容工具函数
 *
 * 后端返回的 status 可能是纯字符串 ("Healthy") 或 tagged union 对象 ({ Healthy: null })。
 * 这些函数统一处理两种格式。
 */
import type { HealthStatus } from '../api/types';

export type HealthLabel = 'Healthy' | 'Degraded' | 'Unhealthy' | 'Unknown';

/** 从 HealthStatus 中提取状态标签 */
export function getHealthLabel(status: HealthStatus | string | undefined | null): HealthLabel {
  if (!status) return 'Unknown';
  if (typeof status === 'string') {
    if (status === 'Healthy' || status === 'Degraded' || status === 'Unhealthy') return status;
    return 'Unknown';
  }
  if ('Healthy' in status) return 'Healthy';
  if ('Degraded' in status) return 'Degraded';
  if ('Unhealthy' in status) return 'Unhealthy';
  return 'Unknown';
}

/** 获取健康状态对应的 CSS 颜色类名 */
export function getHealthColor(status: HealthStatus | string | undefined | null): string {
  const label = getHealthLabel(status);
  switch (label) {
    case 'Healthy': return 'bg-green-500';
    case 'Degraded': return 'bg-amber-500';
    case 'Unhealthy': return 'bg-md-error';
    default: return 'bg-md-outline';
  }
}

/** 获取故障原因（如无则返回 null） */
export function getHealthReason(status: HealthStatus | string | undefined | null): string | null {
  if (!status || typeof status === 'string') return null;
  if ('Degraded' in status && status.Degraded?.reason) return status.Degraded.reason;
  if ('Unhealthy' in status && status.Unhealthy?.reason) return status.Unhealthy.reason;
  return null;
}
