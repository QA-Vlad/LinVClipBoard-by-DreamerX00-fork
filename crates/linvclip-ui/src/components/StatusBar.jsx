function StatusBar({ total, status, onClearAll }) {
    const formatBytes = (bytes) => {
        if (!bytes) return "0 B";
        if (bytes < 1024) return `${bytes} B`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
        return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    };

    return (
        <div className="status-bar">
            <span className="status-item">
                <span className="status-dot"></span>
                Items: {total}
            </span>
            {status && (
                <span className="status-item">
                    Size: {formatBytes(status.db_size_bytes)}
                </span>
            )}
            {total > 0 && (
                <button className="clear-all-btn" onClick={onClearAll} title="Clear all non-pinned items">
                    🗑️ Clear All
                </button>
            )}
            <span className="status-hint">
                <kbd>↑↓</kbd> navigate <kbd>⏎</kbd> paste <kbd>Esc</kbd> close
            </span>
        </div>
    );
}

export default StatusBar;
