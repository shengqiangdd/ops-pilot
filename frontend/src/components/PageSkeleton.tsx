import { cn } from '../lib/cn';

function Bar({ className }: { className?: string }) {
  return <div className={cn('rounded bg-md-surface-container-highest/60 animate-pulse', className)} />;
}

export function ListPageSkeleton() {
  return (
    <div className="space-y-4 animate-slide-up">
      <Bar className="h-8 w-48" />
      <div className="flex gap-3"><Bar className="h-10 w-48" /><Bar className="h-10 w-32" /><Bar className="h-10 w-32" /></div>
      <div className="rounded-md-xl border border-md-outline-variant/50 overflow-hidden">
        {Array.from({ length: 8 }).map((_, i) => (
          <div key={i} className="flex items-center gap-4 px-4 py-3 border-b border-md-outline-variant/30 last:border-0">
            <Bar className="h-4 w-4 shrink-0 rounded" />
            <Bar className="h-4 flex-1" />
            <Bar className="h-4 w-20" />
            <Bar className="h-4 w-24" />
            <Bar className="h-4 w-16" />
          </div>
        ))}
      </div>
    </div>
  );
}

export function DetailPageSkeleton() {
  return (
    <div className="space-y-4 animate-slide-up">
      <Bar className="h-8 w-40" />
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div className="glass-card rounded-md-xl p-5 space-y-3">
          <Bar className="h-6 w-32" />
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="flex justify-between"><Bar className="h-4 w-24" /><Bar className="h-4 w-40" /></div>
          ))}
        </div>
        <div className="glass-card rounded-md-xl p-5 space-y-3">
          <Bar className="h-6 w-32" />
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="flex justify-between"><Bar className="h-4 w-24" /><Bar className="h-4 w-40" /></div>
          ))}
        </div>
      </div>
      <div className="glass-card rounded-md-xl p-5 space-y-3">
        <Bar className="h-6 w-32" />
        <Bar className="h-40 w-full" />
      </div>
    </div>
  );
}

export function ChartPageSkeleton() {
  return (
    <div className="min-h-[calc(100vh-4rem)] p-6" style={{ background: 'linear-gradient(135deg, #0f172a 0%, #1e293b 100%)' }}>
      <div className="flex items-center justify-between mb-6">
        <div className="space-y-2"><Bar className="h-8 w-64" /><Bar className="h-4 w-48" /></div>
      </div>
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={i} className="rounded-xl p-5 border border-gray-700/50" style={{ background: 'rgba(30, 41, 59, 0.8)' }}>
            <Bar className="h-4 w-20 mb-3" />
            <Bar className="h-9 w-24 mb-2" />
            <Bar className="h-1 w-full rounded-full" />
          </div>
        ))}
      </div>
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 rounded-xl p-5 border border-gray-700/50" style={{ background: 'rgba(30, 41, 59, 0.8)' }}>
          <Bar className="h-5 w-32 mb-4" />
          <Bar className="h-64 w-full" />
        </div>
        <div className="space-y-4">
          <div className="rounded-xl p-5 border border-gray-700/50" style={{ background: 'rgba(30, 41, 59, 0.8)' }}>
            <Bar className="h-5 w-32 mb-4" />
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="flex items-center gap-3 mb-3">
                <Bar className="h-2.5 w-2.5 rounded-full shrink-0" />
                <Bar className="h-4 flex-1" />
                <Bar className="h-3 w-12" />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export function FormPageSkeleton() {
  return (
    <div className="space-y-4 animate-slide-up">
      <Bar className="h-8 w-40" />
      <div className="glass-card rounded-md-xl p-5 space-y-4">
        <Bar className="h-6 w-40" />
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div className="space-y-1"><Bar className="h-4 w-20" /><Bar className="h-12 w-full" /></div>
          <div className="space-y-1"><Bar className="h-4 w-20" /><Bar className="h-12 w-full" /></div>
        </div>
        <div className="space-y-1"><Bar className="h-4 w-24" /><Bar className="h-16 w-full" /></div>
        <div className="space-y-1"><Bar className="h-4 w-32" /><Bar className="h-10 w-full" /></div>
        <Bar className="h-10 w-32 rounded-md-lg" />
      </div>
    </div>
  );
}
