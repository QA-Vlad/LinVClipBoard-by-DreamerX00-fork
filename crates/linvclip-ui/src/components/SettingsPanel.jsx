import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function SettingsPanel({ onClose }) {
    const [config, setConfig] = useState(null);
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState(null);

    useEffect(() => {
        loadConfig();
    }, []);

    const loadConfig = async () => {
        try {
            const cfg = await invoke("get_config");
            setConfig(cfg);
        } catch (err) {
            setError("Failed to load settings: " + err);
        }
    };

    const saveConfig = async () => {
        setSaving(true);
        setError(null);
        try {
            await invoke("save_config", { config });
            onClose();
        } catch (err) {
            setError("Save failed: " + err);
        } finally {
            setSaving(false);
        }
    };

    if (!config) {
        return (
            <div className="settings-panel" role="dialog" aria-label="Settings" aria-modal="true">
                <div className="settings-loading">{error || "Loading…"}</div>
            </div>
        );
    }

    return (
        <div className="settings-panel" role="dialog" aria-label="Settings" aria-modal="true">
            <div className="settings-header">
                <h2 id="settings-title">⚙️ Settings</h2>
                <button className="settings-close" onClick={onClose} aria-label="Close settings">
                    ✕
                </button>
            </div>

            <div className="settings-body" role="form" aria-labelledby="settings-title">
                {error && <div className="settings-error" role="alert">{error}</div>}

                <fieldset className="settings-group">
                    <legend>Daemon</legend>
                    <label htmlFor="poll-interval">Poll interval (ms)</label>
                    <input
                        id="poll-interval"
                        type="number"
                        min="50"
                        max="5000"
                        step="50"
                        value={config.daemon?.poll_interval_ms ?? 250}
                        onChange={(e) =>
                            setConfig({
                                ...config,
                                daemon: { ...config.daemon, poll_interval_ms: parseInt(e.target.value) || 250 },
                            })
                        }
                    />
                    <label htmlFor="log-level">Log level</label>
                    <select
                        id="log-level"
                        value={config.daemon?.log_level ?? "info"}
                        onChange={(e) =>
                            setConfig({
                                ...config,
                                daemon: { ...config.daemon, log_level: e.target.value },
                            })
                        }
                    >
                        <option value="trace">Trace</option>
                        <option value="debug">Debug</option>
                        <option value="info">Info</option>
                        <option value="warn">Warn</option>
                        <option value="error">Error</option>
                    </select>
                </fieldset>

                <fieldset className="settings-group">
                    <legend>Storage</legend>
                    <label htmlFor="max-items">Max items</label>
                    <input
                        id="max-items"
                        type="number"
                        min="100"
                        max="100000"
                        value={config.storage?.max_items ?? 10000}
                        onChange={(e) =>
                            setConfig({
                                ...config,
                                storage: { ...config.storage, max_items: parseInt(e.target.value) || 10000 },
                            })
                        }
                    />
                    <label htmlFor="expiry-days">Expiry (days)</label>
                    <input
                        id="expiry-days"
                        type="number"
                        min="1"
                        max="365"
                        value={config.storage?.expiry_days ?? 30}
                        onChange={(e) =>
                            setConfig({
                                ...config,
                                storage: { ...config.storage, expiry_days: parseInt(e.target.value) || 30 },
                            })
                        }
                    />
                    <label htmlFor="max-size">Max item size (MB)</label>
                    <input
                        id="max-size"
                        type="number"
                        min="1"
                        max="200"
                        value={Math.round((config.storage?.max_item_size_bytes ?? 52428800) / 1048576)}
                        onChange={(e) =>
                            setConfig({
                                ...config,
                                storage: {
                                    ...config.storage,
                                    max_item_size_bytes: (parseInt(e.target.value) || 50) * 1048576,
                                },
                            })
                        }
                    />
                </fieldset>

                <fieldset className="settings-group">
                    <legend>UI</legend>
                    <label htmlFor="theme-select">Theme</label>
                    <select
                        id="theme-select"
                        value={config.ui?.theme ?? "dark"}
                        onChange={(e) =>
                            setConfig({ ...config, ui: { ...config.ui, theme: e.target.value } })
                        }
                    >
                        <option value="auto">Auto (follow OS)</option>
                        <option value="dark">Dark</option>
                        <option value="light">Light</option>
                    </select>
                    <label htmlFor="shortcut-input">Global shortcut</label>
                    <input
                        id="shortcut-input"
                        type="text"
                        value={config.ui?.shortcut ?? "Super+."}
                        onChange={(e) =>
                            setConfig({ ...config, ui: { ...config.ui, shortcut: e.target.value } })
                        }
                    />
                </fieldset>

                <fieldset className="settings-group">
                    <legend>Security</legend>
                    <label className="checkbox-label">
                        <input
                            type="checkbox"
                            checked={config.security?.incognito ?? false}
                            onChange={(e) =>
                                setConfig({
                                    ...config,
                                    security: { ...config.security, incognito: e.target.checked },
                                })
                            }
                        />
                        Incognito mode (pause capture)
                    </label>
                    <label htmlFor="blacklist">Blacklisted apps (comma-separated)</label>
                    <input
                        id="blacklist"
                        type="text"
                        value={(config.security?.blacklisted_apps ?? []).join(", ")}
                        onChange={(e) =>
                            setConfig({
                                ...config,
                                security: {
                                    ...config.security,
                                    blacklisted_apps: e.target.value
                                        .split(",")
                                        .map((s) => s.trim())
                                        .filter(Boolean),
                                },
                            })
                        }
                    />
                </fieldset>
            </div>

            <div className="settings-footer">
                <button className="settings-cancel" onClick={onClose}>
                    Cancel
                </button>
                <button className="settings-save" onClick={saveConfig} disabled={saving}>
                    {saving ? "Saving…" : "💾 Save"}
                </button>
            </div>
        </div>
    );
}

export default SettingsPanel;
