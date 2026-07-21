import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { Report, ReportSchedule } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

type View = 'list' | 'preview';

export function ReportsPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [view, setView] = useState<View>('list');
  const [reports, setReports] = useState<Report[]>([]);
  const [schedules, setSchedules] = useState<ReportSchedule[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedReport, setSelectedReport] = useState<Report | null>(null);
  const [showGenerateForm, setShowGenerateForm] = useState(false);
  const [generating, setGenerating] = useState(false);

  // Generate form state
  const [reportType, setReportType] = useState('daily');
  const [selectedSections, setSelectedSections] = useState(['summary', 'resources', 'alerts']);

  const loadReports = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const data = await api.listReports(token);
      setReports(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load reports');
    } finally {
      setLoading(false);
    }
  }, [token]);

  const loadSchedules = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.listReportSchedules(token);
      setSchedules(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load schedules');
    }
  }, [token]);

  useEffect(() => { loadReports(); loadSchedules(); }, [loadReports, loadSchedules]);

  const handleGenerate = async () => {
    setGenerating(true);
    try {
      const report = await api.generateReport(token!, {
        report_type: reportType,
        include_sections: selectedSections,
      });
      setSelectedReport(report);
      setView('preview');
      setShowGenerateForm(false);
      await loadReports();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to generate report');
    } finally {
      setGenerating(false);
    }
  };

  const handleViewReport = async (reportId: string) => {
    try {
      const report = await api.getReport(token!, reportId);
      setSelectedReport(report);
      setView('preview');
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load report');
    }
  };

  const handleExport = async (reportId: string) => {
    window.open(`/api/reports/${reportId}/export`, '_blank');
  };

  const handleScheduleReport = async (enabled: boolean) => {
    try {
      await api.createReportSchedule(token!, {
        enabled,
        report_type: reportType,
        recipients: ['admin@opspilot.local'],
      });
      await loadSchedules();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to schedule report');
    }
  };

  const toggleSection = (section: string) => {
    setSelectedSections(prev =>
      prev.includes(section)
        ? prev.filter(s => s !== section)
        : [...prev, section]
    );
  };

  const sections = [
    { id: 'summary', label: t('reports.section.summary'), icon: '📊' },
    { id: 'resources', label: t('reports.section.resources'), icon: '🖥️' },
    { id: 'alerts', label: t('reports.section.alerts'), icon: '🔔' },
    { id: 'changes', label: t('reports.section.changes'), icon: '📝' },
    { id: 'diagnostics', label: t('reports.section.diagnostics'), icon: '🔍' },
    { id: 'health', label: t('reports.section.health'), icon: '❤️' },
  ];


  if (loading) return <LoadingState skeleton="chart" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {view === 'preview' && selectedReport ? selectedReport.title : t('title.reports')}
        </h2>
        <div className="flex gap-2">
          {view === 'preview' ? (
            <button
              onClick={() => { setView('list'); setSelectedReport(null); }}
              className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high transition-colors"
            >
              {t('reports.back_to_list')}
            </button>
          ) : (
            <>
              <button
                onClick={loadReports}
                disabled={loading}
                className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors"
              >
                {loading ? t('reports.loading') : t('reports.reload')}
              </button>
              <button
                onClick={() => setShowGenerateForm(!showGenerateForm)}
                className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all"
              >
                {showGenerateForm ? t('reports.cancel') : t('reports.generate')}
              </button>
            </>
          )}
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium flex items-center justify-between">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="text-sm underline">{t('reports.dismiss')}</button>
        </div>
      )}

      {view === 'list' && (
        <>
          {/* Generate Form */}
          {showGenerateForm && (
            <div className="glass-card rounded-md-xl p-5 animate-slide-up">
              <h3 className="text-title-medium font-semibold text-md-on-surface mb-4">{t('reports.generate_new')}</h3>

              <div className="space-y-4">
                {/* Report Type */}
                <div>
                  <label className="block text-label-large text-md-on-surface mb-2">{t('reports.type')}</label>
                  <div className="flex gap-2">
                    {['daily', 'weekly', 'monthly'].map((type) => (
                      <button
                        key={type}
                        onClick={() => setReportType(type)}
                        className={cn(
                          'px-4 py-2 text-sm rounded-md-full transition-colors',
                          reportType === type
                            ? 'bg-md-primary text-md-on-primary'
                            : 'bg-md-surface-container-high text-md-on-surface-variant hover:bg-md-surface-container-highest',
                        )}
                      >
                        {t(`reports.type.${type}`)}
                      </button>
                    ))}
                  </div>
                </div>

                {/* Sections */}
                <div>
                  <label className="block text-label-large text-md-on-surface mb-2">{t('reports.sections')}</label>
                  <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
                    {sections.map((section) => (
                      <label
                        key={section.id}
                        className={cn(
                          'flex items-center gap-2 px-3 py-2 rounded-md-lg cursor-pointer transition-colors',
                          selectedSections.includes(section.id)
                            ? 'bg-md-primary-container/30 border border-md-primary/30'
                            : 'bg-md-surface-container-high border border-transparent hover:bg-md-surface-container-highest',
                        )}
                      >
                        <input
                          type="checkbox"
                          checked={selectedSections.includes(section.id)}
                          onChange={() => toggleSection(section.id)}
                          className="w-4 h-4 rounded border-md-outline text-md-primary focus:ring-md-primary"
                        />
                        <span>{section.icon} {section.label}</span>
                      </label>
                    ))}
                  </div>
                </div>

                {/* Actions */}
                <div className="flex items-center gap-3 pt-2">
                  <button
                    onClick={handleGenerate}
                    disabled={generating || selectedSections.length === 0}
                    className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50 flex items-center gap-2"
                  >
                    {generating ? (
                      <div className="h-4 w-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                    ) : (
                      <span>📄</span>
                    )}
                    {generating ? t('reports.generating') : t('reports.generate_btn')}
                  </button>
                  <button
                    onClick={() => handleScheduleReport(true)}
                    className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2.5 text-sm font-medium hover:bg-md-surface-container-high transition-colors"
                  >
                    {t('reports.schedule')}
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* Reports List */}
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {reports.map((report) => (
              <div
                key={report.id}
                className="glass-card rounded-md-xl p-4 cursor-pointer hover:shadow-md-2 transition-all"
                onClick={() => handleViewReport(report.id)}
              >
                <div className="flex items-start justify-between mb-2">
                  <span className={cn(
                    'text-xs font-medium px-2 py-1 rounded-md-sm',
                    report.report_type === 'daily' ? 'bg-blue-500/10 text-blue-600' :
                    report.report_type === 'weekly' ? 'bg-purple-500/10 text-purple-600' :
                    'bg-amber-500/10 text-amber-600',
                  )}>
                    {t(`reports.type.${report.report_type}`)}
                  </span>
                  <button
                    onClick={(e) => { e.stopPropagation(); handleExport(report.id); }}
                    className="text-md-primary text-label-large hover:bg-md-primary-container/30 px-2 py-1 rounded-md-sm transition-colors"
                  >
                    {t('reports.export')}
                  </button>
                </div>
                <h3 className="text-body-medium font-medium text-md-on-surface mb-1">{report.title}</h3>
                <p className="text-body-small text-md-on-surface-variant line-clamp-2">{report.summary}</p>
                <p className="text-label-small text-md-on-surface-variant mt-2">
                  {new Date(report.created_at).toLocaleString()}
                </p>
              </div>
            ))}
            {!loading && reports.length === 0 && (
              <div className="col-span-full glass-card rounded-md-xl p-8 text-center">
                <div className="text-4xl mb-3">📄</div>
                <p className="text-body-medium text-md-on-surface-variant">{t('reports.no_reports')}</p>
              </div>
            )}
          </div>

          {/* Schedules */}
          {schedules.length > 0 && (
            <div className="glass-card rounded-md-xl p-5">
              <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('reports.schedules')}</h3>
              <div className="space-y-2">
                {schedules.map((schedule) => (
                  <div key={schedule.id} className="flex items-center justify-between px-3 py-2 rounded-md-lg bg-md-surface-container-highest/50">
                    <div className="flex items-center gap-3">
                      <span className={cn(
                        'h-2 w-2 rounded-full',
                        schedule.enabled ? 'bg-green-500' : 'bg-md-outline',
                      )} />
                      <span className="text-body-medium text-md-on-surface">{t(`reports.type.${schedule.report_type}`)}</span>
                    </div>
                    <span className="text-label-small text-md-on-surface-variant">
                      {schedule.enabled ? t('reports.enabled') : t('reports.disabled')}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </>
      )}

      {/* Preview */}
      {view === 'preview' && selectedReport && (
        <div className="glass-card rounded-md-xl overflow-hidden">
          <div className="p-4 border-b border-md-outline-variant flex items-center justify-between">
            <div>
              <h3 className="text-title-medium font-semibold text-md-on-surface">{selectedReport.title}</h3>
              <p className="text-body-small text-md-on-surface-variant">
                {new Date(selectedReport.created_at).toLocaleString()}
              </p>
            </div>
            <button
              onClick={() => handleExport(selectedReport.id)}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 transition-all"
            >
              {t('reports.download_html')}
            </button>
          </div>
          <div className="p-4">
            <iframe
              srcDoc={selectedReport.content_html}
              className="w-full rounded-md-lg border border-md-outline-variant"
              style={{ height: '70vh' }}
              title="Report Preview"
            />
          </div>
        </div>
      )}
    </div>
  );
}
