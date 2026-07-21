import { useState } from 'react';
import { api } from '../api/client';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

export function IdsPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();
  const [activeTab, setActiveTab] = useState<'logs' | 'anomaly' | 'ip' | 'blocklist'>('logs');
  const [logLine, setLogLine] = useState('');
  const [logResult, setLogResult] = useState<any>(null);
  const [ipQuery, setIpQuery] = useState('');
  const [ipResult, setIpResult] = useState<any>(null);
  const [blockResult, setBlockResult] = useState<any>(null);
  const [loading, setLoading] = useState(false);

  const handleAnalyzeLog = async () => {
    if (!token || !logLine) return;
    setLoading(true);
    try {
      const data = await api.analyzeLog(token, { log_line: logLine, source: 'ssh' });
      setLogResult(data);
    } catch (e) { console.error(e); }
    setLoading(false);
  };

  const handleIpLookup = async () => {
    if (!token || !ipQuery) return;
    setLoading(true);
    try {
      const [geo, block] = await Promise.all([
        api.geolocateIp(token, ipQuery),
        api.checkBlocklist(token, ipQuery),
      ]);
      setIpResult(geo);
      setBlockResult(block);
    } catch (e) { console.error(e); }
    setLoading(false);
  };

  const tabs = [
    { id: 'logs', label: t('ids.tab_logs') },
    { id: 'anomaly', label: t('ids.tab_anomaly') },
    { id: 'ip', label: t('ids.tab_ip') },
    { id: 'blocklist', label: t('ids.tab_blocklist') },
  ];

  return (
    <div className="space-y-4 animate-slide-up">
      <h2 className="text-headline-small font-medium text-md-on-surface">{t('title.ids')}</h2>

      <div className="flex gap-2 border-b border-md-outline-variant pb-2">
        {tabs.map(tab => (
          <button key={tab.id} onClick={() => setActiveTab(tab.id as any)} className={cn('px-4 py-2 text-sm font-medium rounded-md-lg transition-colors', activeTab === tab.id ? 'bg-md-primary text-md-on-primary' : 'text-md-on-surface-variant hover:bg-md-surface-container-high')}>
            {tab.label}
          </button>
        ))}
      </div>

      {activeTab === 'logs' && (
        <div className="glass-card rounded-md-xl p-5">
          <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('ids.log_analysis')}</h3>
          <textarea value={logLine} onChange={e => setLogLine(e.target.value)} rows={3} className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface font-mono text-sm mb-3" placeholder="Failed password for root from 192.168.1.100 port 22 ssh2" />
          <button onClick={handleAnalyzeLog} disabled={loading} className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 text-sm font-medium hover:shadow-md-2 transition-all disabled:opacity-50">
            {loading ? t('ids.analyzing') : t('ids.analyze_btn')}
          </button>
          {logResult && (
            <div className="mt-4 glass-card rounded-md-lg p-4">
              <div className="flex items-center gap-2 mb-2">
                <span className={cn('px-2 py-1 rounded text-xs font-medium', logResult.is_threat ? 'bg-red-500/10 text-red-600' : 'bg-green-500/10 text-green-600')}>
                  {logResult.is_threat ? t('ids.threat_detected') : t('ids.safe')}
                </span>
                <span className="text-body-small text-md-on-surface-variant">{logResult.description}</span>
              </div>
              {logResult.source_ip && <p className="text-label-small text-md-on-surface-variant">Source IP: {logResult.source_ip}</p>}
            </div>
          )}
        </div>
      )}

      {activeTab === 'ip' && (
        <div className="glass-card rounded-md-xl p-5">
          <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('ids.ip_lookup')}</h3>
          <div className="flex gap-3 mb-3">
            <input value={ipQuery} onChange={e => setIpQuery(e.target.value)} className="flex-1 bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" placeholder="192.168.1.1" />
            <button onClick={handleIpLookup} disabled={loading} className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 text-sm font-medium hover:shadow-md-2 transition-all disabled:opacity-50">
              {t('ids.lookup_btn')}
            </button>
          </div>
          {ipResult && (
            <div className="glass-card rounded-md-lg p-4 space-y-2">
              <div className="grid grid-cols-2 gap-3 text-sm">
                <div><span className="text-label-small text-md-on-surface-variant">Country:</span> <span className="text-md-on-surface">{ipResult.country}</span></div>
                <div><span className="text-label-small text-md-on-surface-variant">City:</span> <span className="text-md-on-surface">{ipResult.city}</span></div>
                <div><span className="text-label-small text-md-on-surface-variant">ISP:</span> <span className="text-md-on-surface">{ipResult.isp}</span></div>
                <div><span className="text-label-small text-md-on-surface-variant">Tor:</span> <span className="text-md-on-surface">{ipResult.is_tor ? 'Yes' : 'No'}</span></div>
              </div>
              {blockResult && (
                <div className={cn('mt-2 px-3 py-2 rounded-md text-sm', blockResult.blocked ? 'bg-red-500/10 text-red-600' : 'bg-green-500/10 text-green-600')}>
                  {blockResult.blocked ? t('ids.blocked') : t('ids.not_blocked')}
                  {blockResult.reason && <span className="ml-2">— {blockResult.reason}</span>}
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {activeTab === 'anomaly' && (
        <div className="glass-card rounded-md-xl p-5">
          <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('ids.anomaly_detection')}</h3>
          <p className="text-body-medium text-md-on-surface-variant">{t('ids.anomaly_hint')}</p>
        </div>
      )}

      {activeTab === 'blocklist' && (
        <div className="glass-card rounded-md-xl p-5">
          <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('ids.blocklist')}</h3>
          <p className="text-body-medium text-md-on-surface-variant">{t('ids.blocklist_hint')}</p>
        </div>
      )}
    </div>
  );
}
