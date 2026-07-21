import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';

interface AuditLogParams {
  page?: number;
  per_page?: number;
  action?: string;
  user?: string;
  from?: string;
  to?: string;
}

export function useAuditLogs(token: string | null, params: AuditLogParams = {}) {
  return useQuery({
    queryKey: ['auditLogs', params],
    queryFn: () => api.listAuditLogs(token!, params as Record<string, string>),
    enabled: !!token,
    staleTime: 1 * 60 * 1000, // 1 minute
  });
}
