import { useTranslation } from "../i18n/index.jsx";
import { getCurrentWindow } from "@tauri-apps/api/window";

function AppHeader({ onOpenSettings, onClearAll, showPreview, onTogglePreview }) {
    const { t } = useTranslation();

    const handleHide = () => getCurrentWindow().hide();

    return (
        <div className="app-header" data-tauri-drag-region>
            <div className="app-header-left" data-tauri-drag-region>
                <span className="app-header-icon" aria-hidden="true">📋</span>
                <h1 className="app-header-title" data-tauri-drag-region>{t("app.title")}</h1>
            </div>
            <div className="app-header-actions">
                <button
                    className={`header-btn${showPreview ? " header-btn-active" : ""}`}
                    onClick={onTogglePreview}
                    title={t("preview.toggle")}
                    aria-label={t("preview.toggle")}
                    aria-pressed={showPreview}
                >
                    👁
                </button>
                <button
                    className="header-btn"
                    onClick={onOpenSettings}
                    title={t("settings.title")}
                    aria-label={t("settings.title")}
                >
                    ⚙️
                </button>
                <button
                    className="header-btn header-btn-danger"
                    onClick={onClearAll}
                    title={t("confirm.clear_all")}
                    aria-label={t("confirm.clear_all")}
                >
                    🗑️
                </button>
                <button
                    className="header-btn header-btn-close"
                    onClick={handleHide}
                    title={t("app.hide_to_tray")}
                    aria-label={t("app.hide_to_tray")}
                >
                    ✕
                </button>
            </div>
        </div>
    );
}

export default AppHeader;
