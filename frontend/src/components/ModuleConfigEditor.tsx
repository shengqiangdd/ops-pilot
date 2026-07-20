import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { ModuleConfig } from '../api/types';

interface ModuleConfigEditorProps {
  moduleName: string;
  onClose: () => void;
}

function validateJson(text: string): string | null {
  try { JSON.parse(text); return null; } catch (e) { return e instanceof Error ? e.message : 'Invalid JSON'; }
}

export function ModuleConfigEditor({ moduleName, onClose }: ModuleConfigEditorProps) {
  const [raw, setRaw] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validationError, setValidationError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const config = await api.getModuleConfig(moduleName);
      setRaw(JSON.stringify(config, null, 2));
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load config');
      setRaw('{\n\n}');
    } finally { setLoading(false); }
  }, [moduleName]);

  useEffect(() => { load(); }, [load]);

  const handleChange = (value: string) => {
    setRaw(value);
    setValidationError(validateJson(value));
    setSaved(false);
  };

  const handleSave = async () => {
    const err = validateJson(raw);
    if (err) { setValidationError(err); return; }
    setSaving(true);
    setError(null);
    try {
      const config: ModuleConfig = JSON.parse(raw);
      await api.saveModuleConfig(moduleName, config);
      setSaved(true);
      setValidationError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to save config');
    } finally { setSaving(false); }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="flex w-full max-w-2xl flex-col rounded-md-xl bg-md-surface-container shadow-md-4">
        <div className="flex items-center justify-between border-b border-md-outline-variant px-6 py-4">
          <h3 className="text-title-large font-medium text-md-on-surface">Configure: {moduleName}</h3>
          <button onClick={onClose} className="text-md-on-surface-variant hover:text-md-on-surface" aria-label="Close">
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" strokeWidth="1.5" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="flex-1 overflow-auto px-6 py-4">
          {loading ? (
            <div className="py-8 text-center text-body-medium text-md-on-surface-variant">Loading configuration...</div>
          ) : (
            <textarea value={raw} onChange={(e) => handleChange(e.target.value)}
              className={`w-full rounded-md-sm p-3 font-mono text-body-medium min-h-[300px] resize-y focus:outline-none focus:ring-2 focus:ring-md-primary/20
                ${validationError ? 'border border-md-error bg-md-error-container/10' : 'bg-md-surface-container-highest border border-md-outline focus:border-md-primary'}`}
              spellCheck={false} />
          )}
          {validationError && <p className="mt-2 text-body-medium text-md-error">JSON error: {validationError}</p>}
          {error && <p className="mt-2 text-body-medium text-md-error">{error}</p>}
          {saved && <p className="mt-2 text-body-medium text-green-600">Configuration saved successfully.</p>}
        </div>

        <div className="flex items-center justify-end gap-3 border-t border-md-outline-variant px-6 py-4">
          <button onClick={onClose}
            className="border border-md-outline text-md-primary rounded-md-lg px-6 py-2.5 font-medium hover:bg-md-surface-container-high transition-colors">
            Cancel
          </button>
          <button onClick={handleSave} disabled={saving || !!validationError || loading}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
            {saving ? 'Saving...' : 'Save'}
          </button>
        </div>
      </div>
    </div>
  );
}
