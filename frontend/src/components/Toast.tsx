import { useState, useEffect, useCallback, createContext, useContext } from 'react';
import { cn } from '../lib/cn';

type ToastVariant = 'success' | 'error' | 'warning' | 'info';

interface Toast {
  id: string;
  message: string;
  variant: ToastVariant;
  duration?: number;
}

interface ToastContextValue {
  addToast: (message: string, variant?: ToastVariant, duration?: number) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

export function useToast() {
  const ctx = useContext(ToastContext);
  if (!ctx) {
    return { addToast: (_msg: string, _variant?: ToastVariant, _duration?: number) => {} };
  }
  return ctx;
}

const variantStyles: Record<ToastVariant, string> = {
  success: 'bg-md-primary text-md-on-primary',
  error: 'bg-md-error text-md-on-error',
  warning: 'bg-amber-500 text-white',
  info: 'bg-md-secondary text-md-on-secondary',
};

const variantIcons: Record<ToastVariant, string> = {
  success: '✓',
  error: '✕',
  warning: '⚠',
  info: 'ℹ',
};

function ToastItem({ toast, onDismiss }: { toast: Toast; onDismiss: (id: string) => void }) {
  const [exiting, setExiting] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => {
      setExiting(true);
      setTimeout(() => onDismiss(toast.id), 300);
    }, toast.duration ?? 4000);
    return () => clearTimeout(timer);
  }, [toast, onDismiss]);

  return (
    <div
      className={cn(
        'pointer-events-auto flex items-center gap-2 rounded-md-lg px-4 py-3 text-body-medium font-medium shadow-md-2 transition-all duration-300',
        variantStyles[toast.variant],
        exiting ? 'translate-x-4 opacity-0' : 'translate-x-0 opacity-100',
      )}
    >
      <span className="flex h-5 w-5 items-center justify-center rounded-md-full bg-white/20 text-xs font-bold">
        {variantIcons[toast.variant]}
      </span>
      <span className="flex-1">{toast.message}</span>
      <button
        onClick={() => { setExiting(true); setTimeout(() => onDismiss(toast.id), 300); }}
        className="ml-2 text-white/70 hover:text-white"
      >
        ✕
      </button>
    </div>
  );
}

let toastCounter = 0;

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const addToast = useCallback(
    (message: string, variant: ToastVariant = 'info', duration?: number) => {
      const id = `toast-${++toastCounter}`;
      setToasts((prev) => [...prev, { id, message, variant, duration }]);
    },
    [],
  );

  const dismiss = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  return (
    <ToastContext.Provider value={{ addToast }}>
      {children}
      <div className="pointer-events-none fixed bottom-4 right-4 z-50 flex flex-col gap-2">
        {toasts.map((t) => (
          <ToastItem key={t.id} toast={t} onDismiss={dismiss} />
        ))}
      </div>
    </ToastContext.Provider>
  );
}
