import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "../i18n/index.jsx";

/**
 * Modal displaying a generated QR code for clipboard text.
 */
function QrModal({ text, onClose, onToast }) {
    const { t } = useTranslation();
    const [qrDataUrl, setQrDataUrl] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);

    // Generate QR on mount
    useState(() => {
        (async () => {
            try {
                const dataUrl = await invoke("generate_qr_code", { text });
                setQrDataUrl(dataUrl);
            } catch (e) {
                setError(String(e));
            } finally {
                setLoading(false);
            }
        })();
    });

    const handleCopyQr = async () => {
        if (!qrDataUrl) return;
        try {
            // Copy the base64 data URL to clipboard
            await invoke("paste_raw_text", { text: qrDataUrl });
            onToast("✅ QR copied!");
        } catch {
            onToast("❌ Failed");
        }
    };

    return (
        <div className="qr-modal-overlay" onClick={onClose}>
            <div className="qr-modal" onClick={(e) => e.stopPropagation()}>
                <div className="qr-modal-header">
                    <h3>📱 {t("context.qr_code")}</h3>
                    <button className="qr-modal-close" onClick={onClose}>✕</button>
                </div>
                <div className="qr-modal-body">
                    {loading && <div className="qr-loading">Generating…</div>}
                    {error && <div className="qr-error">{error}</div>}
                    {qrDataUrl && (
                        <>
                            <img src={qrDataUrl} alt="QR Code" className="qr-image" />
                            <p className="qr-text-preview">
                                {text.length > 60 ? text.slice(0, 57) + "…" : text}
                            </p>
                        </>
                    )}
                </div>
                {qrDataUrl && (
                    <div className="qr-modal-footer">
                        <button className="qr-btn" onClick={handleCopyQr}>
                            📋 {t("context.copy")}
                        </button>
                        <button className="qr-btn qr-btn-secondary" onClick={onClose}>
                            {t("update.close")}
                        </button>
                    </div>
                )}
            </div>
        </div>
    );
}

export default QrModal;
