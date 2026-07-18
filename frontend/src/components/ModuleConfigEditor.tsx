import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { ModuleConfig } from '../api/types';
import { cn } from '../lib/cn';

interface ModuleConfigEditorProps {
  moduleName: string;
  onClose: () => void;
}

function validateJson(text: string): string | null {
  try {
    JSON.parse(text);
    return null;
  } catch (e) {
    return e instanceof Error ? e.message : 'Invalid JSON';
  }
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
    } finally {
      setLoading(false);
    }
  }, [moduleName]);

  useEffect(() => {
    load();
  }, [load]);

  const handleChange = (value: string) => {
    setRaw(value);
    setValidationError(validateJson(value));
    setSaved(false);
  };

  const handleSave = async () => {
    const err = validateJson(raw);
    if (err) {
      setValidationError(err);
      return;
    }

    setSaving(true);
    setError(null);
    try {
      const config: ModuleConfig = JSON.parse(raw);
      await api.saveModuleConfig(moduleName, config);
      setSaved(true);
      setValidationError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to save config');
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="flex w-full max-w-2xl flex-col rounded-lg bg-white shadow-xl">
        <div className="flex items-center justify-between border-b border-gray-200 px-6 py-4">
          <h3 className="text-lg font-semibold text-gray-900">
            Configure: {moduleName}
          </h3>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600"
            aria-label="Close"
          >
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" strokeWidth="1.5" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="flex-1 overflow-auto px-6 py-4">
          {loading ? (
            <div className="py-8 text-center text-sm text-gray-500">Loading configuration...</div>
          ) : (
            <textarea
              value={raw}
              onChange={(e) => handleChange(e.target.value)}
              className={cn(
                'w-full rounded-md border p-3 font-mono text-sm',
                'min-h-[300px] resize-y focus:outline-none focus:ring-2 focus:ring-blue-500',
                validationError ? 'border-red-300 bg-red-50' : 'border-gray-300 bg-gray-50',
              )}
              spellCheck={false}
            />
          )}

          {validationError && (
            <p className="mt-2 text-sm text-red-600">JSON error: {validationError}</p>
          )}
          {error && (
            <p className="mt-2 text-sm text-red-600">{error}</p>
          )}
          {saved && (
            <p className="mt-2 text-sm text-green-600">Configuration saved successfully.</p>
          )}
        </div>

        <div className="flex items-center justify-end gap-3 border-t border-gray-200 px-6 py-4">
          <button
            onClick={onClose}
            className="rounded-md border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={saving || !!validationError || loading}
            className={cn(
              'rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white',
              'hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2',
              'disabled:cursor-not-allowed disabled:opacity-50',
            )}
          >
            {saving ? 'Saving...' : 'Save'}
          </button>
        </div>
      </div>
    </div>
  );
}
