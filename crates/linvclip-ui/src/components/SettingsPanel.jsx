import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "../i18n/index.jsx";

function SettingsPanel({ onClose }) {
    const { t, lang, setLang, availableLanguages } = useTranslation();
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
            // Sync i18n context with the persisted language from backend config
            if (cfg.ui?.language && cfg.ui.language !== lang) {
                setLang(cfg.ui.language);
            }
        } catch (err) {
            setError(t("settings.title") + ": " + err);
        }
    };

    const handleLanguageChange = (newLang) => {
        // Update i18n context immediately for live preview
        setLang(newLang);
        // Update config state so it gets persisted on save
        setConfig((prev) => ({
            ...prev,
            ui: { ...prev.ui, language: newLang },
        }));
    };

    const saveConfig = async () => {
        setSaving(true);
        setError(null);
        try {
            await invoke("save_config", { config });
            onClose();
        } catch (err) {
            setError(t("settings.title") + ": " + err);
        } finally {
            setSaving(false);
        }
    };

    if (!config) {
        return (
            <div className="settings-panel" role="dialog" aria-label={t("settings.title")} aria-modal="true">
                <div className="settings-loading">{error || "Loading…"}</div>
            </div>
        );
    }

    return (
        <div className="settings-panel" role="dialog" aria-label={t("settings.title")} aria-modal="true">
            <div className="settings-header">
                <h2 id="settings-title">⚙️ {t("settings.title")}</h2>
                <button className="settings-close" onClick={onClose} aria-label={t("settings.title")}>
                    ✕
                </button>
            </div>

            <div className="settings-body" role="form" aria-labelledby="settings-title">
                {error && <div className="settings-error" role="alert">{error}</div>}

                <fieldset className="settings-group">
                    <legend>🌐 {t("settings.language")}</legend>
                    <div className="language-toggle" role="radiogroup" aria-label={t("settings.language")}>
                        {availableLanguages.map((code) => (
                            <button
                                key={code}
                                type="button"
                                role="radio"
                                aria-checked={lang === code}
                                className={`lang-btn${lang === code ? " lang-btn-active" : ""}`}
                                onClick={() => handleLanguageChange(code)}
                            >
                                {code === "pt" ? "Português" : "English"}
                            </button>
                        ))}
                    </div>
                </fieldset>

                <fieldset className="settings-group">
                    <legend>{t("settings.daemon")}</legend>
                    <label htmlFor="poll-interval">{t("settings.poll_interval")}</label>
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
                    <label htmlFor="log-level">{t("settings.log_level")}</label>
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
                    <legend>{t("settings.storage")}</legend>
                    <label htmlFor="max-items">{t("settings.max_items")}</label>
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
                    <label htmlFor="expiry-days">{t("settings.expiry_days")}</label>
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
                    <label htmlFor="max-size">{t("settings.max_size")}</label>
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
                    <legend>{t("settings.theme")}</legend>
                    <label htmlFor="theme-select">{t("settings.theme")}</label>
                    <select
                        id="theme-select"
                        value={config.ui?.theme ?? "dark"}
                        onChange={(e) =>
                            setConfig({ ...config, ui: { ...config.ui, theme: e.target.value } })
                        }
                    >
                        <option value="auto">{t("settings.theme_auto")}</option>
                        <option value="dark">{t("settings.theme_dark")}</option>
                        <option value="light">{t("settings.theme_light")}</option>
                    </select>
                    <label htmlFor="shortcut-input">{t("settings.keyboard_shortcuts")}</label>
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
                    <legend>{t("settings.security")}</legend>
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
                        {t("settings.incognito")}
                    </label>
                    <label htmlFor="blacklist">{t("settings.blacklisted_apps")}</label>
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
                    {t("confirm.cancel")}
                </button>
                <button className="settings-save" onClick={saveConfig} disabled={saving}>
                    {saving ? "…" : "💾 Save"}
                </button>
            </div>
        </div>
    );
}

export default SettingsPanel;
