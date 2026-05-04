import { useEffect, useState } from 'react';
import { Save, Download, Upload, X } from 'lucide-react';
import { useCodexUiStore } from '../stores/useCodexUiStore';
import { codexClient } from '../services/codexClient';

function asInteger(value: unknown, fallback: number): number {
  if (typeof value === 'number' && Number.isFinite(value)) return Math.trunc(value);
  if (typeof value === 'string' && value.trim()) {
    const numeric = Number(value);
    if (Number.isFinite(numeric)) return Math.trunc(numeric);
  }
  return fallback;
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

function readBool(config: Record<string, unknown>, key: string, fallback: boolean): boolean {
  const value = config[key];
  return typeof value === 'boolean' ? value : fallback;
}

export function CodexSettingsPage() {
  const store = useCodexUiStore();
  const [autoRefreshMinutes, setAutoRefreshMinutes] = useState('60');
  const [appPath, setAppPath] = useState('');
  const [launchOnSwitch, setLaunchOnSwitch] = useState(false);
  const [localAccessEntryVisible, setLocalAccessEntryVisible] = useState(true);
  const [autoSwitchThreshold, setAutoSwitchThreshold] = useState('20');
  const [quotaAlertThreshold, setQuotaAlertThreshold] = useState('15');
  const [message, setMessage] = useState<{ text: string; tone?: string } | null>(null);
  const [importJsonInput, setImportJsonInput] = useState('');
  const [importMessage, setImportMessage] = useState<string | null>(null);
  const [loadedConfig, setLoadedConfig] = useState<Record<string, unknown>>({});
  const [loadingConfig, setLoadingConfig] = useState(true);
  const [savingConfig, setSavingConfig] = useState(false);

  useEffect(() => {
    let active = true;
    codexClient.getGeneralConfig()
      .then((config) => {
        if (!active) return;
        setLoadedConfig(config);
        setAutoRefreshMinutes(String(asInteger(config.codex_auto_refresh_minutes, 60)));
        setAppPath(typeof config.codex_app_path === 'string' ? config.codex_app_path : '');
        setLaunchOnSwitch(readBool(config, 'codex_launch_on_switch', false));
        setLocalAccessEntryVisible(readBool(config, 'codex_local_access_entry_visible', true));
        setAutoSwitchThreshold(String(asInteger(config.codex_auto_switch_primary_threshold, 20)));
        setQuotaAlertThreshold(String(asInteger(config.codex_quota_alert_threshold, 15)));
      })
      .catch((e) => {
        if (active) setMessage({ text: `Failed to load settings: ${String(e)}`, tone: 'error' });
      })
      .finally(() => {
        if (active) setLoadingConfig(false);
      });
    return () => { active = false; };
  }, []);

  const handleSaveSettings = async () => {
    setSavingConfig(true);
    try {
      const nextConfig = {
        ...loadedConfig,
        codex_auto_refresh_minutes: clamp(asInteger(autoRefreshMinutes, 60), -1, 999),
        codex_app_path: appPath.trim(),
        codex_launch_on_switch: launchOnSwitch,
        codex_local_access_entry_visible: localAccessEntryVisible,
        codex_auto_switch_primary_threshold: clamp(asInteger(autoSwitchThreshold, 20), 0, 100),
        codex_quota_alert_threshold: clamp(asInteger(quotaAlertThreshold, 15), 0, 100),
      };
      await codexClient.saveGeneralConfig(nextConfig);
      setLoadedConfig(nextConfig);
      setMessage({ text: 'Settings saved', tone: 'success' });
      window.setTimeout(() => setMessage(null), 2000);
    } catch (e) {
      setMessage({ text: `Save failed: ${String(e)}`, tone: 'error' });
    } finally {
      setSavingConfig(false);
    }
  };

  const handleExportAll = async () => {
    try {
      const accountsJson = await store.exportAccounts(store.accounts.map(a => a.id));
      const bundle = {
        schema: 'oauthcodex.ui.bundle.v1',
        exportedAt: new Date().toISOString(),
        generalConfig: loadedConfig,
        accounts: JSON.parse(accountsJson),
      };
      const blob = new Blob([JSON.stringify(bundle, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `oauthcodex_bundle_${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(url);
      setMessage({ text: 'Export downloaded', tone: 'success' });
    } catch (e) {
      setMessage({ text: String(e), tone: 'error' });
    }
  };

  const handleImportConfig = async () => {
    if (!importJsonInput.trim()) return;
    try {
      const parsed = JSON.parse(importJsonInput);
      if (parsed && typeof parsed === 'object' && parsed.schema === 'oauthcodex.ui.bundle.v1') {
        if (parsed.accounts) await store.importFromJson(JSON.stringify(parsed.accounts));
        if (parsed.generalConfig && typeof parsed.generalConfig === 'object') {
          await codexClient.saveGeneralConfig(parsed.generalConfig as Record<string, unknown>);
          setLoadedConfig(parsed.generalConfig as Record<string, unknown>);
        }
      } else {
        await store.importFromJson(importJsonInput);
      }
      setImportMessage('Import successful');
      setImportJsonInput('');
    } catch (e) {
      setImportMessage(`Import failed: ${e}`);
    }
  };

  return (
    <div className="codex-settings-page">
      <div className="page-header">
        <h1>Codex Settings</h1>
      </div>

      {message && (
        <div className={`message-bar ${message.tone === 'error' ? 'error' : 'success'}`}>
          {message.text}
          <button className="message-close" onClick={() => setMessage(null)}><X size={14} /></button>
        </div>
      )}

      <div className="settings-section">
        <h2>Auto Refresh</h2>
        <div className="settings-row">
          <label>Auto refresh interval (minutes, -1 to disable)</label>
          <input type="number" className="form-input settings-input" value={autoRefreshMinutes}
            onChange={e => setAutoRefreshMinutes(e.target.value)} min={-1} max={999} />
        </div>
      </div>

      <div className="settings-section">
        <h2>App Path & Launch</h2>
        <div className="settings-row">
          <label>Codex App Path</label>
          <input type="text" className="form-input settings-input" value={appPath}
            onChange={e => setAppPath(e.target.value)} placeholder="/Applications/Codex.app" />
        </div>
        <div className="settings-row">
          <label className="checkbox-label">
            <input type="checkbox" checked={launchOnSwitch} onChange={e => setLaunchOnSwitch(e.target.checked)} />
            Launch Codex on account switch
          </label>
        </div>
      </div>

      <div className="settings-section">
        <h2>Local Access</h2>
        <div className="settings-row">
          <label className="checkbox-label">
            <input type="checkbox" checked={localAccessEntryVisible} onChange={e => setLocalAccessEntryVisible(e.target.checked)} />
            Show Local API Service entry
          </label>
        </div>
      </div>

      <div className="settings-section">
        <h2>Auto-Switch & Alerts</h2>
        <div className="settings-row">
          <label>Auto-switch when quota below (%)</label>
          <input type="number" className="form-input settings-input" value={autoSwitchThreshold}
            onChange={e => setAutoSwitchThreshold(e.target.value)} min={0} max={100} />
        </div>
        <div className="settings-row">
          <label>Quota alert threshold (%)</label>
          <input type="number" className="form-input settings-input" value={quotaAlertThreshold}
            onChange={e => setQuotaAlertThreshold(e.target.value)} min={0} max={100} />
        </div>
      </div>

      <div className="settings-actions">
        <button className="btn btn-primary" onClick={handleSaveSettings} disabled={savingConfig}>
          <Save size={14} /> {savingConfig ? 'Saving...' : loadingConfig ? 'Loading...' : 'Save Settings'}
        </button>
      </div>

      <div className="settings-section">
        <h2>Import / Export</h2>
        <div className="settings-row">
          <button className="btn btn-secondary" onClick={handleExportAll}>
            <Download size={14} /> Export Codex Bundle
          </button>
        </div>
        <div className="settings-row">
          <label>Import from JSON</label>
          <textarea className="form-textarea" value={importJsonInput}
            onChange={e => setImportJsonInput(e.target.value)}
            placeholder="Paste an oauthcodex bundle or account JSON" rows={4} />
          <button className="btn btn-secondary" onClick={handleImportConfig} disabled={!importJsonInput.trim()}>
            <Upload size={14} /> Import
          </button>
          {importMessage && <div className="form-info">{importMessage}</div>}
        </div>
      </div>
    </div>
  );
}
