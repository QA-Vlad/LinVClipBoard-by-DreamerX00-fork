import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { useTranslation } from "../i18n/index.jsx";

function SettingsPanel({ onClose, zoom, onZoomChange }) {
    const { t, lang, setLang, availableLanguages } = useTranslation();
    const [config, setConfig] = useState(null);
    const [error, setError] = useState(null);
    const [advancedOpen, setAdvancedOpen] = useState(false);
    const configRef = useRef(null);
    const dirtyRef = useRef(false);
    const [updateStatus, setUpdateStatus] = useState(null); // null | 'checking' | {has_update, ...} | {error}

    useEffect(() => {
        loadConfig();
    }, []);

    const loadConfig = async () => {
        try {
            const cfg = await invoke("get_config");
            setConfig(cfg);
            configRef.current = cfg;
            if (cfg.ui?.language && cfg.ui.language !== lang) {
                setLang(cfg.ui.language);
            }
        } catch (err) {
            setError(String(err));
        }
    };

    // Auto-save on close
    const autoSave = useCallback(async () => {
        if (!dirtyRef.current || !configRef.current) return;
        try {
            await invoke("save_config", { config: configRef.current });
        } catch (err) {
            console.error("Auto-save failed:", err);
        }
    }, []);

    const handleClose = useCallback(() => {
        autoSave().then(() => onClose());
    }, [autoSave, onClose]);

    const updateConfig = (updater) => {
        dirtyRef.current = true;
        setConfig((prev) => {
            const next = updater(prev);
            configRef.current = next;
            return next;
        });
    };

    const handleLanguageChange = (newLang) => {
        setLang(newLang);
        updateConfig((prev) => ({
            ...prev,
            ui: { ...prev.ui, language: newLang },
        }));
    };

    const handlePositionChange = (mode) => {
        updateConfig((prev) => ({
            ...prev,
            ui: { ...prev.ui, window_position: mode },
        }));
    };

    const handleZoomChange = (val) => {
        const z = Math.max(50, Math.min(200, parseInt(val) || 100));
        if (onZoomChange) onZoomChange(z);
        updateConfig((prev) => ({
            ...prev,
            ui: { ...prev.ui, zoom: z },
        }));
    };

    const handleThemeChange = (newTheme) => {
        updateConfig((prev) => ({
            ...prev,
            ui: { ...prev.ui, theme: newTheme },
        }));
        document.documentElement.setAttribute("data-theme", newTheme === "auto" ? "dark" : newTheme);
        localStorage.setItem("theme", newTheme);
    };

    const handleCheckUpdate = useCallback(async () => {
        setUpdateStatus("checking");
        try {
            const info = await invoke("check_for_updates");
            setUpdateStatus(info);
        } catch (err) {
            setUpdateStatus({ error: String(err) });
        }
    }, []);

    const handleOpenRelease = useCallback(async (url) => {
        try {
            await open(url);
        } catch (_) {
            window.open(url, "_blank");
        }
    }, []);

    if (!config) {
        return (
            <div className="settings-overlay" onClick={handleClose}>
                <div className="settings-panel" role="dialog" aria-label={t("settings.title")} aria-modal="true" onClick={(e) => e.stopPropagation()}>
                    <div className="settings-loading">{error || "Loading…"}</div>
                </div>
            </div>
        );
    }

    const currentZoom = zoom ?? config.ui?.zoom ?? 100;
    const currentPosition = config.ui?.window_position ?? "mouse";

    return (
        <div className="settings-overlay" onClick={handleClose}>
            <div className="settings-panel" role="dialog" aria-label={t("settings.title")} aria-modal="true" onClick={(e) => e.stopPropagation()}>
                <div className="settings-header">
                    <h2 id="settings-title">⚙️ {t("settings.title")}</h2>
                    <button className="settings-close" onClick={handleClose} aria-label="Close">✕</button>
                </div>

                <div className="settings-body" role="form" aria-labelledby="settings-title">
                    {error && <div className="settings-error" role="alert">{error}</div>}

                    {/* ── 🌐 Language ── */}
                    <div className="settings-section">
                        <div className="settings-section-label">🌐 {t("settings.language")}</div>
                        <select
                            className="language-select"
                            value={lang}
                            onChange={(e) => handleLanguageChange(e.target.value)}
                            aria-label={t("settings.language")}
                        >
                            {availableLanguages.map((code) => (
                                <option key={code} value={code}>
                                    {{ en: "English", pt: "Português", ja: "日本語", hi: "हिन्दी" }[code] || code}
                                </option>
                            ))}
                        </select>
                    </div>

                    {/* ── 🎨 Theme ── */}
                    <div className="settings-section">
                        <div className="settings-section-label">🎨 {t("settings.theme")}</div>
                        <div className="toggle-group" role="radiogroup" aria-label={t("settings.theme")}>
                            {[
                                { value: "dark", label: t("settings.theme_dark") },
                                { value: "light", label: t("settings.theme_light") },
                                { value: "auto", label: t("settings.theme_auto") },
                            ].map((opt) => (
                                <button
                                    key={opt.value}
                                    type="button"
                                    role="radio"
                                    aria-checked={(config.ui?.theme ?? "dark") === opt.value}
                                    className={`toggle-btn${(config.ui?.theme ?? "dark") === opt.value ? " toggle-btn-active" : ""}`}
                                    onClick={() => handleThemeChange(opt.value)}
                                >
                                    {opt.label}
                                </button>
                            ))}
                        </div>
                    </div>

                    {/* ── 🖥️ Window Position ── */}
                    <div className="settings-section">
                        <div className="settings-section-label">🖥️ {t("settings.window_position")}</div>
                        <div className="toggle-group" role="radiogroup" aria-label={t("settings.window_position")}>
                            <button
                                type="button"
                                role="radio"
                                aria-checked={currentPosition === "fixed"}
                                className={`toggle-btn${currentPosition === "fixed" ? " toggle-btn-active" : ""}`}
                                onClick={() => handlePositionChange("fixed")}
                            >
                                {t("settings.position_fixed")}
                            </button>
                            <button
                                type="button"
                                role="radio"
                                aria-checked={currentPosition === "mouse"}
                                className={`toggle-btn${currentPosition === "mouse" ? " toggle-btn-active" : ""}`}
                                onClick={() => handlePositionChange("mouse")}
                            >
                                {t("settings.position_mouse")}
                            </button>
                        </div>
                    </div>

                    {/* ── 🔍 Size (Zoom) ── */}
                    <div className="settings-section">
                        <div className="settings-section-label">🔍 {t("settings.size_zoom")}</div>
                        <div className="zoom-slider-row">
                            <input
                                type="range"
                                min="50"
                                max="200"
                                step="10"
                                value={currentZoom}
                                onChange={(e) => handleZoomChange(e.target.value)}
                                className="zoom-slider"
                                aria-label={t("settings.size_zoom")}
                            />
                            <span className="zoom-value">{currentZoom}%</span>
                        </div>
                    </div>

                    {/* ── ⌨️ Keyboard Shortcuts ── */}
                    <div className="settings-section">
                        <div className="settings-section-label">⌨️ {t("settings.keyboard_shortcuts")}</div>
                        <div className="shortcuts-info">
                            <p className="shortcuts-instruction">{t("settings.shortcut_instructions")}</p>
                            <ol className="shortcuts-steps">
                                <li>{t("settings.shortcut_step1")}</li>
                                <li>{t("settings.shortcut_step2")}</li>
                                <li>{t("settings.shortcut_step3")}</li>
                            </ol>
                            <div className="shortcuts-keys">
                                <kbd>Ctrl</kbd>+<kbd>+</kbd> / <kbd>Ctrl</kbd>+<kbd>-</kbd> Zoom
                                &nbsp;&middot;&nbsp;
                                <kbd>Ctrl</kbd>+<kbd>0</kbd> Reset
                            </div>
                        </div>
                    </div>

                    {/* ── Advanced (collapsible) ── */}
                    <div className="settings-advanced">
                        <button
                            type="button"
                            className="settings-advanced-toggle"
                            onClick={() => setAdvancedOpen((o) => !o)}
                            aria-expanded={advancedOpen}
                        >
                            <span className={`advanced-chevron${advancedOpen ? " open" : ""}`}>▶</span>
                            {t("settings.advanced")}
                        </button>

                        {advancedOpen && (
                            <div className="settings-advanced-body">
                                {/* Daemon */}
                                <div className="advanced-group">
                                    <div className="advanced-group-title">{t("settings.daemon")}</div>
                                    <label htmlFor="poll-interval">{t("settings.poll_interval")}</label>
                                    <input
                                        id="poll-interval"
                                        type="number"
                                        min="50"
                                        max="5000"
                                        step="50"
                                        value={config.daemon?.poll_interval_ms ?? 250}
                                        onChange={(e) =>
                                            updateConfig((prev) => ({
                                                ...prev,
                                                daemon: { ...prev.daemon, poll_interval_ms: parseInt(e.target.value) || 250 },
                                            }))
                                        }
                                    />
                                    <label htmlFor="log-level">{t("settings.log_level")}</label>
                                    <select
                                        id="log-level"
                                        value={config.daemon?.log_level ?? "info"}
                                        onChange={(e) =>
                                            updateConfig((prev) => ({
                                                ...prev,
                                                daemon: { ...prev.daemon, log_level: e.target.value },
                                            }))
                                        }
                                    >
                                        <option value="trace">Trace</option>
                                        <option value="debug">Debug</option>
                                        <option value="info">Info</option>
                                        <option value="warn">Warn</option>
                                        <option value="error">Error</option>
                                    </select>
                                </div>

                                {/* Storage */}
                                <div className="advanced-group">
                                    <div className="advanced-group-title">{t("settings.storage")}</div>
                                    <label htmlFor="max-items">{t("settings.max_items")}</label>
                                    <input
                                        id="max-items"
                                        type="number"
                                        min="100"
                                        max="100000"
                                        value={config.storage?.max_items ?? 10000}
                                        onChange={(e) =>
                                            updateConfig((prev) => ({
                                                ...prev,
                                                storage: { ...prev.storage, max_items: parseInt(e.target.value) || 10000 },
                                            }))
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
                                            updateConfig((prev) => ({
                                                ...prev,
                                                storage: { ...prev.storage, expiry_days: parseInt(e.target.value) || 30 },
                                            }))
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
                                            updateConfig((prev) => ({
                                                ...prev,
                                                storage: {
                                                    ...prev.storage,
                                                    max_item_size_bytes: (parseInt(e.target.value) || 50) * 1048576,
                                                },
                                            }))
                                        }
                                    />
                                </div>

                                {/* Security */}
                                <div className="advanced-group">
                                    <div className="advanced-group-title">{t("settings.security")}</div>
                                    <label className="checkbox-label">
                                        <input
                                            type="checkbox"
                                            checked={config.security?.incognito ?? false}
                                            onChange={(e) =>
                                                updateConfig((prev) => ({
                                                    ...prev,
                                                    security: { ...prev.security, incognito: e.target.checked },
                                                }))
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
                                            updateConfig((prev) => ({
                                                ...prev,
                                                security: {
                                                    ...prev.security,
                                                    blacklisted_apps: e.target.value
                                                        .split(",")
                                                        .map((s) => s.trim())
                                                        .filter(Boolean),
                                                },
                                            }))
                                        }
                                    />
                                </div>
                            </div>
                        )}
                    </div>

                    <div className="settings-save-hint">{t("settings.close_to_save")}</div>

                    {/* ── 🔄 Check for Updates ── */}
                    <div className="settings-section settings-update-section">
                        <div className="settings-section-label">🔄 {t("settings.updates")}</div>
                        <div className="settings-update-row">
                            <span className="settings-version">v{updateStatus?.current_version || "1.4.4"}</span>
                            <button
                                className="settings-update-btn"
                                onClick={handleCheckUpdate}
                                disabled={updateStatus === "checking"}
                            >
                                {updateStatus === "checking" ? t("settings.checking") : t("settings.check_updates")}
                            </button>
                        </div>
                        {updateStatus && updateStatus !== "checking" && !updateStatus.error && (
                            <div className={`settings-update-result ${updateStatus.has_update ? "has-update" : "up-to-date"}`}>
                                {updateStatus.has_update ? (
                                    <>
                                        <span>✨ {t("settings.update_available").replace("{v}", updateStatus.latest_version)}</span>
                                        <button className="settings-update-link" onClick={() => handleOpenRelease(updateStatus.release_url)}>
                                            {t("settings.download")}
                                        </button>
                                    </>
                                ) : (
                                    <span>✅ {t("settings.up_to_date")}</span>
                                )}
                            </div>
                        )}
                        {updateStatus?.error && (
                            <div className="settings-update-result update-error">⚠️ {updateStatus.error}</div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}

export default SettingsPanel;
