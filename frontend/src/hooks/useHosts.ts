import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';

export function useHosts(token: string | null) {
  return useQuery({
    queryKey: ['hosts'],
    queryFn: () => api.listHosts(token!),
    enabled: !!token,
    staleTime: 2 * 60 * 1000, // 2 minutes
  });
}
