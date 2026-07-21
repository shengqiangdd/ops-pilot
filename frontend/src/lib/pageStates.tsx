import { ListPageSkeleton, DetailPageSkeleton, ChartPageSkeleton, FormPageSkeleton } from '../components/PageSkeleton';

const skeletons = {
  list: ListPageSkeleton,
  detail: DetailPageSkeleton,
  chart: ChartPageSkeleton,
  form: FormPageSkeleton,
};

export function LoadingState({ skeleton }: { skeleton: 'list' | 'detail' | 'chart' | 'form' }) {
  const Component = skeletons[skeleton];
  return <Component />;
}

export function EmptyState({ icon, title, description, action }: { icon: string; title: string; description: string; action?: React.ReactNode }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 animate-slide-up">
      <span className="text-5xl mb-4">{icon}</span>
      <h3 className="text-title-large font-semibold text-md-on-surface mb-1">{title}</h3>
      <p className="text-body-medium text-md-on-surface-variant mb-4">{description}</p>
      {action}
    </div>
  );
}

export function ErrorState({ message, onRetry }: { message: string; onRetry?: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 animate-slide-up">
      <span className="text-5xl mb-4">⚠️</span>
      <h3 className="text-title-large font-semibold text-md-on-surface mb-1">出错了</h3>
      <p className="text-body-medium text-md-on-surface-variant mb-4">{message}</p>
      {onRetry && (
        <button onClick={onRetry} className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 text-sm font-medium hover:shadow-md-2 transition-all">
          重试
        </button>
      )}
    </div>
  );
}
