import { useState, useEffect, useCallback, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import ClipboardList from "./components/ClipboardList";
import SearchBar from "./components/SearchBar";
import StatusBar from "./components/StatusBar";

function App() {
    const [items, setItems] = useState([]);
    const [total, setTotal] = useState(0);
    const [searchQuery, setSearchQuery] = useState("");
    const [loading, setLoading] = useState(false);
    const [status, setStatus] = useState(null);
    const [selectedIndex, setSelectedIndex] = useState(0);
    const [toast, setToast] = useState(null);
    const offset = useRef(0);
    const LIMIT = 30;

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

        // Auto-refresh every 2 seconds
        const interval = setInterval(() => {
            if (!searchQuery.trim()) {
                fetchItems(true);
            }
        }, 2000);

        return () => clearInterval(interval);
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
        <div className="app">
            <div className="app-container">
                <SearchBar value={searchQuery} onChange={setSearchQuery} />

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

                <StatusBar total={total} status={status} onClearAll={handleClearAll} />
            </div>
            {toast && <div className="copy-toast">{toast}</div>}
        </div>
    );
}

export default App;
