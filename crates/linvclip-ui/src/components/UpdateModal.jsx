import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-shell";
import { useTranslation } from "../i18n/index.jsx";

/**
 * UpdateModal — shown when a new version is available.
 *
 * Props:
 *   updateInfo  – the UpdateInfo object from check_for_updates
 *   onClose     – callback to dismiss the modal
 */
function UpdateModal({ updateInfo, onClose }) {
    const { t } = useTranslation();
    // "idle" | "downloading" | "done" | "error"
    const [stage, setStage] = useState("idle");
    const [progress, setProgress] = useState({ downloaded: 0, total: 0, percent: 0 });
    const [savedPath, setSavedPath] = useState("");
    const [errorMsg, setErrorMsg] = useState("");

    /* Listen to download-progress events from the Rust backend */
    useEffect(() => {
        let unlisten;
        (async () => {
            unlisten = await listen("download-progress", (event) => {
                setProgress(event.payload);
            });
        })();
        return () => { if (unlisten) unlisten(); };
    }, []);

    const handleDownload = useCallback(async () => {
        setStage("downloading");
        setProgress({ downloaded: 0, total: 0, percent: 0 });
        try {
            const path = await invoke("download_update", {
                url: updateInfo.deb_download_url,
                version: updateInfo.latest_version,
            });
            setSavedPath(path);
            setStage("done");
        } catch (err) {
            setErrorMsg(String(err));
            setStage("error");
        }
    }, [updateInfo]);

    const handleVisitGithub = useCallback(async () => {
        try {
            await open(updateInfo.release_url);
        } catch (_) {
            window.open(updateInfo.release_url, "_blank");
        }
    }, [updateInfo]);

    /* Format bytes for display */
    const fmtBytes = (b) => {
        if (b < 1024) return `${b} B`;
        if (b < 1048576) return `${(b / 1024).toFixed(1)} KB`;
        return `${(b / 1048576).toFixed(1)} MB`;
    };

    /* Render simple Markdown-ish release notes: headings, bold, bullets, code */
    const renderNotes = (md) => {
        if (!md) return null;
        return md.split("\n").map((line, i) => {
            // headings
            if (line.startsWith("### ")) return <h4 key={i}>{line.slice(4)}</h4>;
            if (line.startsWith("## ")) return <h3 key={i}>{line.slice(3)}</h3>;
            // bullet
            if (line.startsWith("- ")) {
                const content = line.slice(2)
                    .replace(/\*\*(.+?)\*\*/g, "<b>$1</b>")
                    .replace(/`(.+?)`/g, "<code>$1</code>");
                return <li key={i} dangerouslySetInnerHTML={{ __html: content }} />;
            }
            // empty line
            if (!line.trim()) return <div key={i} className="update-notes-spacer" />;
            // plain text with bold/code
            const content = line
                .replace(/\*\*(.+?)\*\*/g, "<b>$1</b>")
                .replace(/`(.+?)`/g, "<code>$1</code>");
            return <p key={i} dangerouslySetInnerHTML={{ __html: content }} />;
        });
    };

    return (
        <div className="update-modal-overlay" onClick={onClose}>
            <div
                className="update-modal"
                role="dialog"
                aria-modal="true"
                aria-label={t("update.title")}
                onClick={(e) => e.stopPropagation()}
            >
                {/* Header */}
                <div className="update-modal-header">
                    <span className="update-modal-icon">🚀</span>
                    <div>
                        <h2 className="update-modal-title">{t("update.title")}</h2>
                        <span className="update-modal-version">
                            v{updateInfo.current_version} → v{updateInfo.latest_version}
                        </span>
                    </div>
                </div>

                {/* Release notes */}
                <div className="update-notes-scroll">
                    <div className="update-notes">
                        {renderNotes(updateInfo.release_notes)}
                    </div>
                </div>

                {/* Progress bar (visible during/after download) */}
                {stage === "downloading" && (
                    <div className="update-progress-section">
                        <div className="update-progress-track">
                            <div
                                className="update-progress-fill"
                                style={{ width: `${progress.percent}%` }}
                            />
                        </div>
                        <div className="update-progress-text">
                            {fmtBytes(progress.downloaded)} / {progress.total > 0 ? fmtBytes(progress.total) : "…"}{" "}
                            <span className="update-progress-pct">{progress.percent.toFixed(0)}%</span>
                        </div>
                    </div>
                )}

                {stage === "done" && (
                    <div className="update-done-msg">
                        ✅ {t("update.saved_to")} <code>{savedPath}</code>
                        <p className="update-install-hint">{t("update.install_hint")}</p>
                    </div>
                )}

                {stage === "error" && (
                    <div className="update-error-msg">⚠️ {errorMsg}</div>
                )}

                {/* Action buttons */}
                <div className="update-modal-actions">
                    {stage === "idle" && (
                        <>
                            <button className="update-btn-primary" onClick={handleDownload} disabled={!updateInfo.deb_download_url}>
                                ⬇ {t("update.download_now")}
                            </button>
                            <button className="update-btn-secondary" onClick={handleVisitGithub}>
                                🌐 {t("update.visit_github")}
                            </button>
                            <button className="update-btn-ghost" onClick={onClose}>
                                {t("update.cancel")}
                            </button>
                        </>
                    )}
                    {stage === "downloading" && (
                        <button className="update-btn-ghost" disabled>
                            {t("update.downloading")}
                        </button>
                    )}
                    {stage === "done" && (
                        <>
                            <button className="update-btn-secondary" onClick={handleVisitGithub}>
                                🌐 {t("update.visit_github")}
                            </button>
                            <button className="update-btn-ghost" onClick={onClose}>
                                {t("update.close")}
                            </button>
                        </>
                    )}
                    {stage === "error" && (
                        <>
                            <button className="update-btn-primary" onClick={handleDownload}>
                                🔄 {t("update.retry")}
                            </button>
                            <button className="update-btn-ghost" onClick={onClose}>
                                {t("update.close")}
                            </button>
                        </>
                    )}
                </div>
            </div>
        </div>
    );
}

export default UpdateModal;
