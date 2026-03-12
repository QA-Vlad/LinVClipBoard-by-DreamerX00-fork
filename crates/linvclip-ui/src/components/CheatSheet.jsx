import { useEffect, useRef } from "react";
import { useTranslation } from "../i18n/index.jsx";
import { useKeybindings, KEYBINDING_ACTIONS } from "../contexts/KeybindingContext.jsx";

/**
 * Cheat sheet overlay showing all active keybindings.
 * Triggered by ? (vim) or Ctrl+/ (standard).
 */
function CheatSheet({ onClose }) {
    const { t } = useTranslation();
    const { getBinding, vimMode } = useKeybindings();
    const overlayRef = useRef(null);

    useEffect(() => {
        const handleKey = (e) => {
            if (e.key === "Escape" || e.key === "?") {
                e.preventDefault();
                onClose();
            }
        };
        document.addEventListener("keydown", handleKey, true);
        return () => document.removeEventListener("keydown", handleKey, true);
    }, [onClose]);

    const renderKey = (combo) => {
        if (!combo) return <span className="cs-key cs-key-none">—</span>;
        return combo.split("+").map((part, i) => (
            <span key={i}>
                {i > 0 && <span className="cs-key-sep">+</span>}
                <kbd className="cs-key">{part}</kbd>
            </span>
        ));
    };

    return (
        <div className="cheatsheet-overlay" onClick={onClose} ref={overlayRef}>
            <div className="cheatsheet" onClick={(e) => e.stopPropagation()}>
                <div className="cheatsheet-header">
                    <h2>⌨️ {t("vim.cheatsheet_title")}</h2>
                    {vimMode && <span className="cs-vim-badge">VIM</span>}
                    <button className="cheatsheet-close" onClick={onClose}>✕</button>
                </div>
                <div className="cheatsheet-body">
                    {Object.entries(KEYBINDING_ACTIONS).map(([category, actions]) => (
                        <div key={category} className="cs-category">
                            <h3 className="cs-category-title">{category}</h3>
                            <div className="cs-grid">
                                {actions.map(({ action, label }) => (
                                    <div key={action} className="cs-row">
                                        <span className="cs-action">{t(label)}</span>
                                        <span className="cs-binding">{renderKey(getBinding(action))}</span>
                                    </div>
                                ))}
                            </div>
                        </div>
                    ))}

                    {vimMode && (
                        <div className="cs-category">
                            <h3 className="cs-category-title">Vim Extras</h3>
                            <div className="cs-grid">
                                <div className="cs-row">
                                    <span className="cs-action">{t("keybind.select_first")}</span>
                                    <span className="cs-binding"><kbd className="cs-key">g</kbd><kbd className="cs-key">g</kbd></span>
                                </div>
                                <div className="cs-row">
                                    <span className="cs-action">{t("keybind.delete_selected")}</span>
                                    <span className="cs-binding"><kbd className="cs-key">d</kbd><kbd className="cs-key">d</kbd></span>
                                </div>
                                <div className="cs-row">
                                    <span className="cs-action">Toggle Select</span>
                                    <span className="cs-binding"><kbd className="cs-key">v</kbd></span>
                                </div>
                            </div>
                        </div>
                    )}

                    {/* Zoom */}
                    <div className="cs-category">
                        <h3 className="cs-category-title">Zoom</h3>
                        <div className="cs-grid">
                            <div className="cs-row">
                                <span className="cs-action">Zoom In</span>
                                <span className="cs-binding"><kbd className="cs-key">Ctrl</kbd><span className="cs-key-sep">+</span><kbd className="cs-key">+</kbd></span>
                            </div>
                            <div className="cs-row">
                                <span className="cs-action">Zoom Out</span>
                                <span className="cs-binding"><kbd className="cs-key">Ctrl</kbd><span className="cs-key-sep">+</span><kbd className="cs-key">-</kbd></span>
                            </div>
                            <div className="cs-row">
                                <span className="cs-action">Reset</span>
                                <span className="cs-binding"><kbd className="cs-key">Ctrl</kbd><span className="cs-key-sep">+</span><kbd className="cs-key">0</kbd></span>
                            </div>
                        </div>
                    </div>
                </div>
                <div className="cheatsheet-footer">
                    <span className="cs-hint">Press <kbd>Esc</kbd> to close</span>
                </div>
            </div>
        </div>
    );
}

export default CheatSheet;
