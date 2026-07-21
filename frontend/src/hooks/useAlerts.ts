import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';

export function useAlertRules(token: string | null) {
  return useQuery({
    queryKey: ['alertRules'],
    queryFn: () => api.listAlertRules(token!),
    enabled: !!token,
    staleTime: 2 * 60 * 1000,
  });
}

export function useAlertHistory(token: string | null, params?: Record<string, string>) {
  return useQuery({
    queryKey: ['alertHistory', params],
    queryFn: () => api.listAlertHistory(token!, params),
    enabled: !!token,
    staleTime: 1 * 60 * 1000,
  });
}
