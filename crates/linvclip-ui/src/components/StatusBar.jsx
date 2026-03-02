function StatusBar({ total, status, onClearAll, onOpenSettings, theme, onThemeToggle }) {
    const formatBytes = (bytes) => {
        if (!bytes) return "0 B";
        if (bytes < 1024) return `${bytes} B`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
        return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    };

    return (
        <div className="status-bar" role="toolbar" aria-label="Status and actions">
            <span className="status-item" role="status">
                <span className="status-dot" aria-hidden="true"></span>
                Items: {total}
            </span>
            {status && (
                <span className="status-item" role="status">
                    Size: {formatBytes(status.db_size_bytes)}
                </span>
            )}
            {total > 0 && (
                <button
                    className="clear-all-btn"
                    onClick={onClearAll}
                    title="Clear all non-pinned items"
                    aria-label="Clear all non-pinned items"
                >
                    🗑️ Clear All
                </button>
            )}
            <button
                className="theme-toggle-btn"
                onClick={onThemeToggle}
                title={`Switch to ${theme === "dark" ? "light" : "dark"} theme`}
                aria-label={`Switch to ${theme === "dark" ? "light" : "dark"} theme`}
            >
                {theme === "dark" ? "☀️" : "🌙"}
            </button>
            <button
                className="settings-btn"
                onClick={onOpenSettings}
                title="Settings"
                aria-label="Open settings"
            >
                ⚙️
            </button>
            <span className="status-hint" aria-hidden="true">
                <kbd>↑↓</kbd> navigate <kbd>⏎</kbd> paste <kbd>Esc</kbd> close
            </span>
        </div>
    );
}

export default StatusBar;
