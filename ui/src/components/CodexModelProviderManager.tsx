import { useState, useEffect, useMemo } from 'react';
import { Plus, Search, Pencil, Trash2, X } from 'lucide-react';
import { useCodexUiStore, type CodexModelProvider } from '../stores/useCodexUiStore';
import { maskApiKey } from '../types/codex';

interface CodexModelProviderManagerProps {
  onProvidersChanged?: (providers: CodexModelProvider[]) => void;
  onManageModelPresets?: () => void;
}

function normalizeBaseUrl(value: string): string {
  return value.trim().replace(/\/+$/, '');
}

function isHttpUrl(value: string): boolean {
  try {
    const url = new URL(value);
    return url.protocol === 'http:' || url.protocol === 'https:';
  } catch {
    return false;
  }
}

export function CodexModelProviderManager({ onProvidersChanged }: CodexModelProviderManagerProps) {
  const store = useCodexUiStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [showModal, setShowModal] = useState(false);
  const [editProviderId, setEditProviderId] = useState<string | null>(null);
  const [formName, setFormName] = useState('');
  const [formBaseUrl, setFormBaseUrl] = useState('');
  const [formWebsite, setFormWebsite] = useState('');
  const [formApiKeyUrl, setFormApiKeyUrl] = useState('');
  const [formNewApiKey, setFormNewApiKey] = useState('');
  const [formNewApiKeyName, setFormNewApiKeyName] = useState('');
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [notice, setNotice] = useState<{ text: string; tone: 'success' | 'error' } | null>(null);

  useEffect(() => { store.fetchModelProviders(); }, []);

  useEffect(() => {
    onProvidersChanged?.(store.modelProviders);
  }, [onProvidersChanged, store.modelProviders]);

  const filteredProviders = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    return q ? store.modelProviders.filter(p => p.name.toLowerCase().includes(q) || p.baseUrl.toLowerCase().includes(q)) : store.modelProviders;
  }, [store.modelProviders, searchQuery]);

  const openCreate = () => {
    setEditProviderId(null);
    setFormName(''); setFormBaseUrl(''); setFormWebsite(''); setFormApiKeyUrl('');
    setFormNewApiKey(''); setFormNewApiKeyName('');
    setFormError(null); setShowModal(true);
  };

  const openEdit = (provider: CodexModelProvider) => {
    setEditProviderId(provider.id);
    setFormName(provider.name); setFormBaseUrl(provider.baseUrl);
    setFormWebsite(provider.website || ''); setFormApiKeyUrl(provider.apiKeyUrl || '');
    setFormNewApiKey(''); setFormNewApiKeyName('');
    setFormError(null); setShowModal(true);
  };

  const handleSave = async () => {
    const name = formName.trim();
    const baseUrl = normalizeBaseUrl(formBaseUrl);
    const apiKey = formNewApiKey.trim();
    const apiKeyName = formNewApiKeyName.trim() || undefined;
    if (!name || !baseUrl) { setFormError('Name and Base URL required'); return; }
    if (!isHttpUrl(baseUrl)) { setFormError('Base URL must be a valid http(s) URL'); return; }
    if (!editProviderId && !apiKey) { setFormError('Initial API Key required'); return; }
    const duplicateProvider = store.modelProviders.some(p => p.id !== editProviderId && (p.name.trim().toLowerCase() === name.toLowerCase() || normalizeBaseUrl(p.baseUrl).toLowerCase() === baseUrl.toLowerCase()));
    if (duplicateProvider) { setFormError('A provider with this name or Base URL already exists'); return; }
    if (editProviderId && apiKey) {
      const provider = store.modelProviders.find(p => p.id === editProviderId);
      if (provider?.apiKeys.some(k => k.apiKey.trim() === apiKey)) {
        setFormError('This API key already exists for the provider');
        return;
      }
    }
    setSaving(true); setFormError(null);
    try {
      if (editProviderId) {
        await store.updateModelProvider(editProviderId, { name, baseUrl, website: formWebsite.trim() || undefined, apiKeyUrl: formApiKeyUrl.trim() || undefined });
        if (apiKey) await store.addModelProviderApiKey(editProviderId, apiKey, apiKeyName);
      } else {
        await store.createModelProvider({ name, baseUrl, website: formWebsite.trim() || undefined, apiKeyUrl: formApiKeyUrl.trim() || undefined, initialApiKey: apiKey, initialApiKeyName: apiKeyName });
      }
      setShowModal(false);
      setNotice({ tone: 'success', text: 'Provider saved' });
    } catch (e) { setFormError(String(e)); }
    finally { setSaving(false); }
  };

  const handleDeleteProvider = async (id: string) => {
    if (!window.confirm('Delete this provider?')) return;
    try { await store.deleteModelProvider(id); setNotice({ tone: 'success', text: 'Provider deleted' }); }
    catch (e) { setNotice({ tone: 'error', text: String(e) }); }
  };

  const handleDeleteKey = async (providerId: string, apiKeyId: string) => {
    try {
      await store.removeModelProviderApiKey(providerId, apiKeyId);
      setNotice({ tone: 'success', text: 'API key removed' });
    } catch (e) {
      setNotice({ tone: 'error', text: String(e) });
    }
  };

  return (
    <div className="codex-provider-manager">
      {notice && (
        <div className={`message-bar ${notice.tone === 'error' ? 'error' : 'success'}`}>
          {notice.text}<button onClick={() => setNotice(null)}><X size={14} /></button>
        </div>
      )}
      <div className="page-toolbar">
        <div className="toolbar-left">
          <div className="search-box"><Search size={16} className="search-icon" />
            <input type="text" placeholder="Search providers..." value={searchQuery} onChange={e => setSearchQuery(e.target.value)} />
          </div>
        </div>
        <div className="toolbar-right">
          <button className="btn btn-primary btn-sm" onClick={openCreate}><Plus size={14} /> Add Provider</button>
        </div>
      </div>
      <div className="provider-grid">
        {filteredProviders.map(provider => (
          <div key={provider.id} className="provider-card">
            <div className="provider-card-header">
              <span className="provider-name">{provider.name}</span>
              <div className="provider-actions">
                <button className="icon-btn" onClick={() => openEdit(provider)}><Pencil size={14} /></button>
                <button className="icon-btn danger" onClick={() => handleDeleteProvider(provider.id)}><Trash2 size={14} /></button>
              </div>
            </div>
            <div className="provider-base-url"><code>{provider.baseUrl}</code></div>
            {provider.apiKeys.length > 0 && (
              <div className="provider-key-list">
                {provider.apiKeys.map(key => (
                  <div key={key.id} className="provider-key-row">
                    <span className="key-name">{key.name || 'Unnamed'}</span>
                    <code className="key-value">{maskApiKey(key.apiKey)}</code>
                    <button className="icon-btn danger" onClick={() => handleDeleteKey(provider.id, key.id)} title="Remove API key">
                      <Trash2 size={12} />
                    </button>
                  </div>
                ))}
              </div>
            )}
            <div className="provider-badge">{provider.apiKeys.length} key(s)</div>
          </div>
        ))}
      </div>
      {filteredProviders.length === 0 && <div className="empty-state"><h3>No providers</h3></div>}

      {showModal && (
        <div className="modal-overlay" onClick={() => !saving && setShowModal(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>{editProviderId ? 'Edit' : 'Add'} Model Provider</h2>
              <button className="modal-close" onClick={() => setShowModal(false)} disabled={saving}><X /></button>
            </div>
            <div className="modal-body">
              <div className="form-group"><label>Name</label><input className="form-input" value={formName} onChange={e => setFormName(e.target.value)} disabled={saving} /></div>
              <div className="form-group"><label>Base URL</label><input className="form-input" value={formBaseUrl} onChange={e => setFormBaseUrl(e.target.value)} placeholder="https://api.openai.com/v1" disabled={saving} /></div>
              <div className="form-group"><label>Website (optional)</label><input className="form-input" value={formWebsite} onChange={e => setFormWebsite(e.target.value)} disabled={saving} /></div>
              <div className="form-group"><label>API Key URL (optional)</label><input className="form-input" value={formApiKeyUrl} onChange={e => setFormApiKeyUrl(e.target.value)} disabled={saving} /></div>
              {editProviderId && (
                <>
                  <div className="form-group"><label>New API Key Name (optional)</label><input className="form-input" value={formNewApiKeyName} onChange={e => setFormNewApiKeyName(e.target.value)} disabled={saving} /></div>
                  <div className="form-group"><label>New API Key (add to existing)</label><input type="password" className="form-input" value={formNewApiKey} onChange={e => setFormNewApiKey(e.target.value)} disabled={saving} /></div>
                </>
              )}
              {!editProviderId && (
                <>
                  <div className="form-group"><label>Initial API Key Name (optional)</label><input className="form-input" value={formNewApiKeyName} onChange={e => setFormNewApiKeyName(e.target.value)} disabled={saving} /></div>
                  <div className="form-group"><label>Initial API Key</label><input type="password" className="form-input" value={formNewApiKey} onChange={e => setFormNewApiKey(e.target.value)} disabled={saving} /></div>
                </>
              )}
              {formError && <div className="form-error">{formError}</div>}
            </div>
            <div className="modal-footer">
              <button className="btn btn-secondary" onClick={() => setShowModal(false)} disabled={saving}>Cancel</button>
              <button className="btn btn-primary" onClick={handleSave} disabled={saving}>{saving ? 'Saving...' : 'Save'}</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
