import { useCallback, useEffect, useRef, useState } from 'react';
import { api } from '../api/client';
import { getHealthLabel, getHealthColor } from '../lib/health';
import { cn } from '../lib/cn';
import { Skeleton, SkeletonCard } from './Skeleton';
import type { ModuleHealth } from '../api/types';

/* ── 数字动画 hook ── */
function useCountUp(target: number, duration = 1200) {
  const [value, setValue] = useState(0);
  const ref = useRef<number | null>(null);

  useEffect(() => {
    const start = performance.now();
    const from = value;
    ref.current = requestAnimationFrame(function tick(now) {
      const elapsed = now - start;
      const progress = Math.min(elapsed / duration, 1);
      const eased = 1 - Math.pow(1 - progress, 3);
      setValue(Math.round(from + (target - from) * eased));
      if (progress < 1) ref.current = requestAnimationFrame(tick);
    });
    return () => { if (ref.current) cancelAnimationFrame(ref.current); };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [target, duration]);

  return value;
}

/* ── 统计卡片 ── */
function StatCard({
  icon, label, value, color, gradient, delay = 0,
}: {
  icon: string; label: string; value: number; color: string; gradient: string; delay?: number;
}) {
  const animated = useCountUp(value);
  return (
    <div
      className="glass-card rounded-md-xl p-5 animate-slide-up"
      style={{ animationDelay: `${delay}ms` }}
    >
      <div className="flex items-start justify-between">
        <div>
          <p className="text-label-medium text-md-on-surface-variant mb-1">{label}</p>
          <p className={cn('text-headline-medium font-bold tabular-nums', color)}>
            {animated}
          </p>
        </div>
        <div className={cn('w-11 h-11 rounded-md-xl flex items-center justify-center text-xl', gradient)}>
          {icon}
        </div>
      </div>
      <div className="mt-3 h-1 rounded-full bg-md-surface-container-highest overflow-hidden">
        <div
          className={cn('h-full rounded-full transition-all duration-1000 ease-out', color.replace('text', 'bg'))}
          style={{ width: `${Math.min((value / (value * 1.5)) * 100, 100)}%` }}
        />
      </div>
    </div>
  );
}

/* ── 模块健康卡片 ── */
function HealthItem({ name, status, enabled, index }: ModuleHealth & { index: number }) {
  const label = getHealthLabel(status) ?? 'Unknown';
  const dot = getHealthColor(status);

  return (
    <div
      className={cn(
        'flex items-center gap-3 px-4 py-3 rounded-md-lg glass-card animate-slide-up',
        !enabled && 'opacity-50',
      )}
      style={{ animationDelay: `${150 + index * 40}ms` }}
    >
      <span className={cn('h-2.5 w-2.5 rounded-full shrink-0', dot)} />
      <span className="flex-1 text-body-medium font-medium text-md-on-surface truncate">{name}</span>
      <span className={cn('text-label-medium', label === 'Healthy' ? 'text-green-500' : label === 'Degraded' ? 'text-amber-500' : 'text-red-500')}>
        {label === 'Healthy' ? '健康' : label === 'Degraded' ? '降级' : label === 'Unhealthy' ? '异常' : label}
      </span>
      <span className={cn('h-2 w-2 rounded-full', enabled ? 'bg-green-500' : 'bg-md-outline')} />
    </div>
  );
}

/* ── 主仪表盘 ── */
export function Dashboard() {
  const [health, setHealth] = useState<ModuleHealth[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.getHealthAll();
      setHealth(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : '加载失败');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); const iv = setInterval(load, 30000); return () => clearInterval(iv); }, [load]);

  const healthy = health.filter(m => getHealthLabel(m.status) === 'Healthy').length;
  const degraded = health.filter(m => getHealthLabel(m.status) === 'Degraded').length;
  const unhealthy = health.filter(m => getHealthLabel(m.status) === 'Unhealthy').length;
  const total = health.length;

  /* 骨架屏 loading */
  if (loading && health.length === 0) {
    return (
      <div className="space-y-6 animate-slide-up">
        <div className="flex items-center justify-between">
          <div>
            <Skeleton height="28px" width="180px" />
            <Skeleton height="14px" width="240px" className="mt-2" />
          </div>
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          {Array.from({ length: 4 }).map((_, i) => <SkeletonCard key={i} />)}
        </div>
        <div className="glass-card rounded-md-xl p-5 space-y-3">
          {Array.from({ length: 6 }).map((_, i) => (
            <div key={i} className="flex items-center gap-3">
              <Skeleton circle height="10px" width="10px" />
              <Skeleton height="14px" width="30%" />
              <Skeleton height="12px" width="50px" className="ml-auto" />
            </div>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6 animate-slide-up">
      {/* 头部 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-headline-medium gradient-text">OpsPilot</h1>
          <p className="text-body-medium text-md-on-surface-variant mt-1">
            AI 驱动的基础设施运维平台 · 实时状态与智能分析
          </p>
        </div>
        <button
          onClick={load}
          disabled={loading}
          className="glass-card rounded-md-lg px-5 py-2.5 text-body-medium font-medium text-md-primary hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50 flex items-center gap-2"
        >
          <svg className={cn('w-4 h-4', loading && 'animate-spin')} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          {loading ? '刷新中...' : '刷新'}
        </button>
      </div>

      {error && (
        <div className="glass-card rounded-md-lg px-5 py-4 text-body-medium text-md-error bg-md-error-container/20">
          {error}
        </div>
      )}

      {/* 统计卡片网格 */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          icon="📦"
          label="模块总数"
          value={total}
          color="text-md-primary"
          gradient="bg-gradient-to-br from-primary/20 to-primary/5"
          delay={0}
        />
        <StatCard
          icon="✅"
          label="健康模块"
          value={healthy}
          color="text-green-500"
          gradient="bg-gradient-to-br from-green-500/20 to-green-500/5"
          delay={80}
        />
        <StatCard
          icon="⚠️"
          label="降级模块"
          value={degraded}
          color="text-amber-500"
          gradient="bg-gradient-to-br from-amber-500/20 to-amber-500/5"
          delay={160}
        />
        <StatCard
          icon="❌"
          label="异常模块"
          value={unhealthy}
          color="text-red-500"
          gradient="bg-gradient-to-br from-red-500/20 to-red-500/5"
          delay={240}
        />
      </div>

      {/* 模块健康列表 + 快速入口 */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 glass-card rounded-md-xl p-5">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-title-medium font-semibold text-md-on-surface">模块健康状态</h2>
            <span className="text-label-medium text-md-on-surface-variant">
              {healthy}/{total} 健康
            </span>
          </div>
          <div className="space-y-2">
            {health.map((m, i) => <HealthItem key={m.name} {...m} index={i} />)}
            {health.length === 0 && (
              <p className="text-body-medium text-md-on-surface-variant text-center py-8">暂无模块数据</p>
            )}
          </div>
        </div>

        <div className="glass-card rounded-md-xl p-5">
          <h2 className="text-title-medium font-semibold text-md-on-surface mb-4">快速入口</h2>
          <div className="grid grid-cols-2 gap-3">
            {[
              { icon: '🖥️', label: '主机管理', tab: 'hosts' },
              { icon: '🛡️', label: '安全扫描', tab: 'security' },
              { icon: '📊', label: '性能监控', tab: 'monitor' },
              { icon: '⏰', label: '任务调度', tab: 'scheduler' },
              { icon: '📚', label: '知识库', tab: 'knowledge' },
              { icon: '💬', label: 'AI 对话', tab: 'chat' },
            ].map((item) => (
              <a
                key={item.tab}
                href={`/${item.tab}`}
                className="flex flex-col items-center gap-2 glass-card rounded-md-lg px-3 py-4 text-md-on-surface-variant hover:text-md-primary hover:border-primary/40 transition-all duration-200 group"
              >
                <span className="text-2xl group-hover:scale-110 transition-transform duration-200">{item.icon}</span>
                <span className="text-label-medium text-center">{item.label}</span>
              </a>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
