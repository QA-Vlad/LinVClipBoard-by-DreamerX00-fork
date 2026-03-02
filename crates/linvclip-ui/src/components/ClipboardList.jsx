import { useRef, useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

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
}) {
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
        const now = new Date();
        const diffMs = now - date;
        const diffSec = Math.floor(diffMs / 1000);
        const diffMin = Math.floor(diffSec / 60);
        const diffHour = Math.floor(diffMin / 60);
        const diffDay = Math.floor(diffHour / 24);

        if (diffSec < 60) return "just now";
        if (diffMin < 60) return `${diffMin}m`;
        if (diffHour < 24) return `${diffHour}h`;
        return `${diffDay}d`;
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

    if (items.length === 0 && !loading) {
        return (
            <div className="empty-state" role="status" aria-label="No clipboard items">
                <span className="empty-icon">📋</span>
                <p className="empty-title">No clipboard items yet</p>
                <p className="empty-sub">Copy something to get started!</p>
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
                    getTypeIcon={getTypeIcon}
                    formatPreview={formatPreview}
                    formatTime={formatTime}
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
    getTypeIcon,
    formatPreview,
    formatTime,
}) {
    const imgSrc = useImagePreview(item);
    const tags = (() => {
        try {
            return JSON.parse(item.tags || "[]");
        } catch {
            return [];
        }
    })();

    return (
        <div
            ref={index === selectedIndex ? selectedRef : null}
            className={`clip-item ${index === selectedIndex ? "selected" : ""} ${
                item.pinned ? "pinned" : ""
            }`}
            onClick={() => onPaste(item.id)}
            role="option"
            aria-selected={index === selectedIndex}
            aria-label={`${getTypeIcon(item.content_type)} ${formatPreview(item.preview_text, 40)}`}
        >
            <div className="clip-item-header">
                <span className="clip-type-icon">{getTypeIcon(item.content_type)}</span>
                {item.pinned && <span className="pin-badge">📌</span>}
                {tags.length > 0 && (
                    <span className="tag-badges">
                        {tags.map((t) => (
                            <span key={t} className="tag-badge">{t}</span>
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
                                alt="Clipboard image"
                                className="clip-thumbnail"
                                loading="lazy"
                            />
                        ) : (
                            <span className="image-dims">{item.preview_text}</span>
                        )}
                    </div>
                ) : (
                    <p className="clip-text">{formatPreview(item.preview_text)}</p>
                )}
            </div>

            <div className="clip-actions">
                <button
                    className="action-btn pin-btn"
                    onClick={(e) => {
                        e.stopPropagation();
                        onPin(item.id);
                    }}
                    title={item.pinned ? "Unpin" : "Pin"}
                    aria-label={item.pinned ? "Unpin item" : "Pin item"}
                >
                    {item.pinned ? "📌" : "📍"}
                </button>
                <button
                    className="action-btn delete-btn"
                    onClick={(e) => {
                        e.stopPropagation();
                        onDelete(item.id);
                    }}
                    title="Delete"
                    aria-label="Delete item"
                >
                    🗑️
                </button>
            </div>
        </div>
    );
}

export default ClipboardList;
