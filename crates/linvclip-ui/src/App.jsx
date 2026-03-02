import { useState, useEffect, useCallback, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import ClipboardList from "./components/ClipboardList";
import SearchBar from "./components/SearchBar";
import StatusBar from "./components/StatusBar";
import SettingsPanel from "./components/SettingsPanel";
import ConfirmDialog from "./components/ConfirmDialog";

function App() {
    const [items, setItems] = useState([]);
    const [total, setTotal] = useState(0);
    const [searchQuery, setSearchQuery] = useState("");
    const [loading, setLoading] = useState(false);
    const [status, setStatus] = useState(null);
    const [selectedIndex, setSelectedIndex] = useState(0);
    const [toast, setToast] = useState(null);
    const [showSettings, setShowSettings] = useState(false);
    const [showConfirm, setShowConfirm] = useState(false);
    const [theme, setTheme] = useState(() => localStorage.getItem("theme") || "dark");
    const offset = useRef(0);
    const LIMIT = 30;

    // Apply theme to DOM (#39)
    useEffect(() => {
        document.documentElement.setAttribute("data-theme", theme);
        localStorage.setItem("theme", theme);
    }, [theme]);

    // Store latest values in refs so the document-level keydown handler
    // always sees current state without needing to re-attach the listener.
    const itemsRef = useRef(items);
    const selectedIndexRef = useRef(selectedIndex);
    useEffect(() => { itemsRef.current = items; }, [items]);
    useEffect(() => { selectedIndexRef.current = selectedIndex; }, [selectedIndex]);

    const fetchItems = useCallback(
        async (reset = false) => {
            if (loading) return;
            setLoading(true);
            try {
                if (reset) {
                    offset.current = 0;
                }

                let result;
                if (searchQuery.trim()) {
                    result = await invoke("search_items", {
                        query: searchQuery,
                        limit: LIMIT,
                    });
                } else {
                    result = await invoke("get_items", {
                        offset: offset.current,
                        limit: LIMIT,
                    });
                }

                if (result) {
                    if (reset || offset.current === 0) {
                        setItems(result.items || []);
                    } else {
                        setItems((prev) => [...prev, ...(result.items || [])]);
                    }
                    setTotal(result.total || 0);
                    offset.current += LIMIT;
                }
            } catch (err) {
                console.error("Failed to fetch items:", err);
            } finally {
                setLoading(false);
            }
        },
        [searchQuery, loading]
    );

    // Initial load
    useEffect(() => {
        fetchItems(true);
        fetchStatus();

        // Listen for push-based clipboard-updated events from the daemon (#13)
        let unlisten;
        const setupListener = async () => {
            unlisten = await listen("clipboard-updated", () => {
                if (!searchQuery.trim()) {
                    fetchItems(true);
                }
                fetchStatus();
            });
        };
        setupListener();

        // Fallback polling every 5s in case events are missed
        const interval = setInterval(() => {
            if (!searchQuery.trim()) {
                fetchItems(true);
            }
            fetchStatus();
        }, 5000);

        return () => {
            clearInterval(interval);
            if (unlisten) unlisten();
        };
    }, []);

    // Re-fetch on search change
    useEffect(() => {
        const timer = setTimeout(() => {
            fetchItems(true);
        }, 300);
        return () => clearTimeout(timer);
    }, [searchQuery]);

    const fetchStatus = async () => {
        try {
            const s = await invoke("get_status");
            setStatus(s);
        } catch (err) {
            console.error("Status fetch failed:", err);
        }
    };

    const showToast = (message) => {
        setToast(message);
        setTimeout(() => setToast(null), 700);
    };

    const hideWindow = async () => {
        try {
            const win = getCurrentWindow();
            await win.hide();
        } catch (err) {
            console.error("Hide failed:", err);
        }
    };

    const handlePaste = async (id) => {
        try {
            await invoke("paste_item", { id });
            showToast("✅ Copied!");
            // Auto-close after showing feedback
            setTimeout(() => hideWindow(), 500);
        } catch (err) {
            showToast("❌ Failed");
            console.error("Paste failed:", err);
        }
    };

    const handlePin = async (id) => {
        try {
            await invoke("pin_item", { id });
            setItems((prev) =>
                prev.map((item) =>
                    item.id === id ? { ...item, pinned: !item.pinned } : item
                )
            );
        } catch (err) {
            console.error("Pin failed:", err);
        }
    };

    const handleDelete = async (id) => {
        try {
            await invoke("delete_item", { id });
            setItems((prev) => prev.filter((item) => item.id !== id));
            setTotal((t) => Math.max(0, t - 1));
        } catch (err) {
            console.error("Delete failed:", err);
        }
    };

    const handleLoadMore = () => {
        if (items.length < total) {
            fetchItems(false);
        }
    };

    const handleClearAll = async () => {
        // Actual clear logic (called after confirmation)
        try {
            await invoke("clear_all");
            showToast("🗑️ Cleared!");
            fetchItems(true);
            fetchStatus();
        } catch (err) {
            // Fallback: delete items one by one
            const unpinned = items.filter(i => !i.pinned);
            for (const item of unpinned) {
                try {
                    await invoke("delete_item", { id: item.id });
                } catch (_) { }
            }
            showToast("🗑️ Cleared!");
            fetchItems(true);
            fetchStatus();
        }
    };

    // Triggered by StatusBar — shows confirmation first (#41)
    const requestClearAll = () => {
        setShowConfirm(true);
    };

    const handleConfirmClear = () => {
        setShowConfirm(false);
        handleClearAll();
    };

    // ─── Global keyboard handler via document listener ───
    // Uses refs so the handler always sees fresh state without re-attaching.
    useEffect(() => {
        const handleKeyDown = (e) => {
            const curItems = itemsRef.current;
            const curIdx = selectedIndexRef.current;

            switch (e.key) {
                case "Escape":
                    e.preventDefault();
                    e.stopPropagation();
                    hideWindow();
                    break;
                case "ArrowDown":
                    e.preventDefault();
                    setSelectedIndex((i) => Math.min(i + 1, curItems.length - 1));
                    break;
                case "ArrowUp":
                    e.preventDefault();
                    setSelectedIndex((i) => Math.max(i - 1, 0));
                    break;
                case "Enter":
                    // Don't capture Enter if user is typing in search
                    if (document.activeElement?.tagName === "INPUT") return;
                    e.preventDefault();
                    if (curItems[curIdx]) {
                        handlePaste(curItems[curIdx].id);
                    }
                    break;
                case "Delete":
                    if (document.activeElement?.tagName === "INPUT") return;
                    if (curItems[curIdx]) {
                        handleDelete(curItems[curIdx].id);
                    }
                    break;
            }
        };

        document.addEventListener("keydown", handleKeyDown, true); // capture phase
        return () => document.removeEventListener("keydown", handleKeyDown, true);
    }, []); // Empty deps — uses refs for state

    return (
        <div className="app" role="application" aria-label="LinVClipBoard">
            <div className="app-container">
                <header role="banner">
                    <SearchBar value={searchQuery} onChange={setSearchQuery} />
                </header>

                <main role="main" aria-label="Clipboard history">
                    <ClipboardList
                        items={items}
                        selectedIndex={selectedIndex}
                        onPaste={handlePaste}
                        onPin={handlePin}
                        onDelete={handleDelete}
                        onLoadMore={handleLoadMore}
                        loading={loading}
                        hasMore={items.length < total}
                    />
                </main>

                <footer role="contentinfo">
                    <StatusBar
                        total={total}
                        status={status}
                        onClearAll={requestClearAll}
                        onOpenSettings={() => setShowSettings(true)}
                        theme={theme}
                        onThemeToggle={() =>
                            setTheme((t) => (t === "dark" ? "light" : "dark"))
                        }
                    />
                </footer>
            </div>

            {toast && <div className="copy-toast" role="alert">{toast}</div>}

            {showSettings && (
                <SettingsPanel onClose={() => setShowSettings(false)} />
            )}

            {showConfirm && (
                <ConfirmDialog
                    message="Delete all non-pinned clipboard items? This cannot be undone."
                    onConfirm={handleConfirmClear}
                    onCancel={() => setShowConfirm(false)}
                />
            )}
        </div>
    );
}

export default App;
