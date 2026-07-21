import { cn } from '../lib/cn';

interface DiagnosticCardProps {
  name: string;
  status: string;
  score: number;
  icon: string;
  onClick?: () => void;
  expanded?: boolean;
}

const STATUS_CONFIG: Record<string, { color: string; bg: string; ring: string }> = {
  ok: { color: 'text-green-500', bg: 'bg-green-500/10', ring: 'stroke-green-500' },
  warning: { color: 'text-amber-500', bg: 'bg-amber-500/10', ring: 'stroke-amber-500' },
  critical: { color: 'text-red-500', bg: 'bg-red-500/10', ring: 'stroke-red-500' },
};

const CATEGORY_ICONS: Record<string, string> = {
  CPU: '🖥️',
  Memory: '💾',
  Disk: '💿',
  Network: '🌐',
  Services: '⚙️',
  Security: '🛡️',
};

export function DiagnosticCard({ name, status, score, icon, onClick, expanded }: DiagnosticCardProps) {
  const config = STATUS_CONFIG[status] || STATUS_CONFIG.ok;
  const displayIcon = CATEGORY_ICONS[name] || icon;

  // SVG circle progress
  const radius = 20;
  const circumference = 2 * Math.PI * radius;
  const progress = (score / 100) * circumference;

  return (
    <button
      onClick={onClick}
      className={cn(
        'glass-card rounded-md-xl p-4 text-left transition-all hover:shadow-md-2 w-full',
        expanded && 'ring-2 ring-md-primary/30',
      )}
    >
      <div className="flex items-center gap-4">
        {/* Score Circle */}
        <div className="relative w-14 h-14 shrink-0">
          <svg className="w-14 h-14 -rotate-90" viewBox="0 0 50 50">
            <circle
              cx="25"
              cy="25"
              r={radius}
              fill="none"
              stroke="var(--md-sys-color-surface-container-highest)"
              strokeWidth="4"
            />
            <circle
              cx="25"
              cy="25"
              r={radius}
              fill="none"
              className={cn('transition-all duration-1000', config.ring)}
              strokeWidth="4"
              strokeDasharray={circumference}
              strokeDashoffset={circumference - progress}
              strokeLinecap="round"
            />
          </svg>
          <div className="absolute inset-0 flex items-center justify-center">
            <span className={cn('text-xs font-bold', config.color)}>
              {Math.round(score)}
            </span>
          </div>
        </div>

        {/* Info */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-lg">{displayIcon}</span>
            <span className="text-title-small font-semibold text-md-on-surface">{name}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className={cn(
              'inline-flex items-center gap-1 text-xs font-medium px-2 py-0.5 rounded-full',
              config.bg, config.color,
            )}>
              <span className={cn('h-1.5 w-1.5 rounded-full', config.ring.replace('stroke', 'bg'))} />
              {status}
            </span>
            <span className="text-label-small text-md-on-surface-variant">
              {score >= 80 ? 'Healthy' : score >= 60 ? 'Warning' : 'Critical'}
            </span>
          </div>
        </div>

        {/* Expand Arrow */}
        <svg
          className={cn('w-4 h-4 text-md-on-surface-variant transition-transform', expanded && 'rotate-180')}
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          strokeWidth={2}
        >
          <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
        </svg>
      </div>
    </button>
  );
}
