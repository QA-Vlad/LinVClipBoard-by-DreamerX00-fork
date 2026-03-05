import { useRef, useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "../i18n/index.jsx";

/** Small hook to lazily load image thumbnails (#38). */
function useImagePreview(item) {
    const [src, setSrc] = useState(null);
    const loaded = useRef(false);

    useEffect(() => {
        if (item.content_type !== "image" || loaded.current) return;
        loaded.current = true;
        invoke("get_image_base64", { path: item.content })
            .then(setSrc)
            .catch(() => {});
    }, [item.content, item.content_type]);

    return src;
}

function ClipboardList({
    items,
    selectedIndex,
    onPaste,
    onPin,
    onDelete,
    onLoadMore,
    loading,
    hasMore,
    onContextMenu,
}) {
    const { t } = useTranslation();
    const listRef = useRef(null);
    const selectedRef = useRef(null);

    // Scroll selected item into view
    useEffect(() => {
        if (selectedRef.current) {
            selectedRef.current.scrollIntoView({
                block: "nearest",
                behavior: "smooth",
            });
        }
    }, [selectedIndex]);

    // Infinite scroll
    const handleScroll = () => {
        if (!listRef.current || loading || !hasMore) return;
        const { scrollTop, scrollHeight, clientHeight } = listRef.current;
        if (scrollHeight - scrollTop - clientHeight < 100) {
            onLoadMore();
        }
    };

    const formatTime = (dateStr) => {
        const date = new Date(dateStr);
        const hh = String(date.getHours()).padStart(2, "0");
        const mm = String(date.getMinutes()).padStart(2, "0");
        const ss = String(date.getSeconds()).padStart(2, "0");
        return `${hh}:${mm}:${ss}`;
    };

    const getTypeIcon = (contentType) => {
        switch (contentType) {
            case "plain_text":
                return "📝";
            case "html":
                return "🌐";
            case "image":
                return "🖼️";
            case "rich_text":
                return "📄";
            case "files":
                return "📁";
            case "uri":
                return "🔗";
            default:
                return "📋";
        }
    };

    const formatPreview = (text, maxLen = 80) => {
        if (!text) return "";
        const clean = text.replace(/\n/g, " ↵ ").replace(/\s+/g, " ").trim();
        if (clean.length > maxLen) return clean.slice(0, maxLen - 3) + "...";
        return clean;
    };

    /** Heuristic: content looks like code if it has braces, semicolons, arrows, etc. */
    const looksLikeCode = (text) => {
        if (!text) return false;
        const codePatterns = /[{};=>]|function |const |let |var |import |def |class |<\/?\w+>/;
        return codePatterns.test(text);
    };

    if (items.length === 0 && !loading) {
        return (
            <div className="empty-state" role="status" aria-label={t("clipboard.empty_title")}>
                <span className="empty-icon">📋</span>
                <p className="empty-title">{t("clipboard.empty_title")}</p>
                <p className="empty-sub">{t("clipboard.empty_sub")}</p>
            </div>
        );
    }

    return (
        <div
            className="clipboard-list"
            ref={listRef}
            onScroll={handleScroll}
            role="listbox"
            aria-label="Clipboard items"
        >
            {items.map((item, index) => (
                <ClipItem
                    key={item.id}
                    item={item}
                    index={index}
                    selectedIndex={selectedIndex}
                    selectedRef={selectedRef}
                    onPaste={onPaste}
                    onPin={onPin}
                    onDelete={onDelete}
                    onContextMenu={onContextMenu}
                    getTypeIcon={getTypeIcon}
                    formatPreview={formatPreview}
                    formatTime={formatTime}
                    looksLikeCode={looksLikeCode}
                    t={t}
                />
            ))}

            {loading && (
                <div className="loading-indicator" role="status" aria-label="Loading more items">
                    <div className="spinner"></div>
                </div>
            )}
        </div>
    );
}

/** Individual clip item – extracted for the image preview hook. */
function ClipItem({
    item,
    index,
    selectedIndex,
    selectedRef,
    onPaste,
    onPin,
    onDelete,
    onContextMenu,
    getTypeIcon,
    formatPreview,
    formatTime,
    looksLikeCode,
    t,
}) {
    const imgSrc = useImagePreview(item);
    const tags = (() => {
        try {
            return JSON.parse(item.tags || "[]");
        } catch {
            return [];
        }
    })();

    const isCode = looksLikeCode(item.preview_text);

    return (
        <div
            ref={index === selectedIndex ? selectedRef : null}
            className={`clip-item ${index === selectedIndex ? "selected" : ""} ${
                item.pinned ? "pinned" : ""
            }`}
            onClick={() => onPaste(item.id)}
            onContextMenu={(e) => {
                e.preventDefault();
                e.stopPropagation();
                if (onContextMenu) onContextMenu(e, item);
            }}
            role="option"
            aria-selected={index === selectedIndex}
            aria-label={`${getTypeIcon(item.content_type)} ${formatPreview(item.preview_text, 40)}`}
        >
            <div className="clip-item-body">
                <div className="clip-item-header">
                    <span className="clip-type-icon">{getTypeIcon(item.content_type)}</span>
                    {item.pinned && <span className="pin-badge">📌</span>}
                    {tags.length > 0 && (
                        <span className="tag-badges">
                            {tags.map((tg) => (
                                <span key={tg} className="tag-badge">{tg}</span>
                            ))}
                        </span>
                    )}
                    <span className="clip-time">{formatTime(item.created_at)}</span>
                </div>

                <div className="clip-content">
                    {item.content_type === "image" ? (
                        <div className="clip-image-preview">
                            {imgSrc ? (
                                <img
                                    src={imgSrc}
                                    alt={t("clipboard.image_preview")}
                                    className="clip-thumbnail"
                                    loading="lazy"
                                />
                            ) : (
                                <span className="image-dims">{item.preview_text}</span>
                            )}
                            <span className="clip-type-label">clipboard-image.png</span>
                        </div>
                    ) : (
                        <p className={`clip-text${isCode ? " clip-text-code" : ""}`}>
                            {formatPreview(item.preview_text)}
                        </p>
                    )}
                </div>

                <div className="clip-actions">
                    <button
                        className="action-btn pin-btn"
                        onClick={(e) => {
                            e.stopPropagation();
                            onPin(item.id);
                        }}
                        title={item.pinned ? t("clipboard.unpin") : t("clipboard.pin")}
                        aria-label={item.pinned ? t("clipboard.unpin") : t("clipboard.pin")}
                    >
                        {item.pinned ? "📌" : "📍"}
                    </button>
                    <button
                        className="action-btn delete-btn"
                        onClick={(e) => {
                            e.stopPropagation();
                            onDelete(item.id);
                        }}
                        title={t("clipboard.delete")}
                        aria-label={t("clipboard.delete")}
                    >
                        🗑️
                    </button>
                </div>
            </div>
        </div>
    );
}

export default ClipboardList;
