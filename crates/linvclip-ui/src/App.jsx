import { useState, useEffect, useCallback, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "./i18n/index.jsx";
import { useZoom } from "./hooks/useZoom.js";
import AppHeader from "./components/AppHeader";
import TabBar from "./components/TabBar";
import SearchBar from "./components/SearchBar";
import ClipboardList from "./components/ClipboardList";
import EmojiPicker from "./components/EmojiPicker";
import SymbolPicker from "./components/SymbolPicker";
import GifPicker from "./components/GifPicker";
import FilterPills from "./components/FilterPills";
import Footer from "./components/Footer";
import SettingsPanel from "./components/SettingsPanel";
import ConfirmDialog from "./components/ConfirmDialog";
import ContextMenu from "./components/ContextMenu";
import QrModal from "./components/QrModal";
import PreviewPane from "./components/PreviewPane";

function App() {
    const { t } = useTranslation();
    const { zoom, setZoom, zoomIn, zoomOut, zoomReset } = useZoom();
    const [activeTab, setActiveTab] = useState("clipboard");
    const [filterType, setFilterType] = useState("all");
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
    const [ctxMenu, setCtxMenu] = useState(null); // { item, x, y }
    const [qrText, setQrText] = useState(null); // text to generate QR for
    const [showPreview, setShowPreview] = useState(() => localStorage.getItem("showPreview") === "true");
    const [previewItem, setPreviewItem] = useState(null);
    const offset = useRef(0);
    const LIMIT = 30;

    // Resolve "auto" theme based on OS preference
    const resolveTheme = (t) => {
        if (t !== "auto") return t;
        return window.matchMedia?.("(prefers-color-scheme: light)")?.matches
            ? "catppuccin-latte"
            : "catppuccin-mocha";
    };

    // Apply theme + accent to DOM
    useEffect(() => {
        document.documentElement.setAttribute("data-theme", resolveTheme(theme));
        localStorage.setItem("theme", theme);
    }, [theme]);

    // Apply accent color override from localStorage
    useEffect(() => {
        const accent = localStorage.getItem("accent_color") || "auto";
        if (accent !== "auto") {
            document.documentElement.style.setProperty("--accent", accent);
            // Compute lighter/darker for hover/active
            document.documentElement.style.setProperty("--accent-hover", accent + "cc");
            document.documentElement.style.setProperty("--accent-active", accent);
            document.documentElement.style.setProperty("--accent-glow", accent + "4d");
            document.documentElement.style.setProperty("--border-focus", accent + "80");
        } else {
            document.documentElement.style.removeProperty("--accent");
            document.documentElement.style.removeProperty("--accent-hover");
            document.documentElement.style.removeProperty("--accent-active");
            document.documentElement.style.removeProperty("--accent-glow");
            document.documentElement.style.removeProperty("--border-focus");
        }
    }, [theme]); // re-apply when theme changes

    // Listen for OS theme changes when in "auto" mode
    useEffect(() => {
        const mq = window.matchMedia?.("(prefers-color-scheme: light)");
        if (!mq) return;
        const handler = () => {
            if (localStorage.getItem("theme") === "auto") {
                document.documentElement.setAttribute("data-theme", resolveTheme("auto"));
            }
        };
        mq.addEventListener("change", handler);
        return () => mq.removeEventListener("change", handler);
    }, []);

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

    // Context menu handler
    const handleContextMenu = useCallback((e, item) => {
        setCtxMenu({ item, x: e.clientX, y: e.clientY });
    }, []);

    // Update a single item in the list (after tag add/remove)
    const handleItemUpdate = useCallback((updated) => {
        setItems((prev) => prev.map((it) => (it.id === updated.id ? updated : it)));
    }, []);

    // Persist preview toggle & auto-resize window
    useEffect(() => {
        localStorage.setItem("showPreview", showPreview);
        // Resize window for comfortable split layout
        (async () => {
            try {
                const win = getCurrentWindow();
                const factor = await win.scaleFactor();
                const size = await win.innerSize();
                const logicalW = size.width / factor;
                if (showPreview && logicalW < 700) {
                    await win.setSize(new (await import("@tauri-apps/api/dpi")).LogicalSize(740, size.height / factor));
                } else if (!showPreview && logicalW > 500) {
                    await win.setSize(new (await import("@tauri-apps/api/dpi")).LogicalSize(420, size.height / factor));
                }
            } catch {}
        })();
    }, [showPreview]);

    // Update preview item when selection changes
    useEffect(() => {
        if (showPreview && filteredItems[selectedIndex]) {
            setPreviewItem(filteredItems[selectedIndex]);
        }
    }, [selectedIndex, showPreview, items, filterType]);

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
                case "ArrowRight":
                    if (document.activeElement?.tagName === "INPUT") return;
                    e.preventDefault();
                    setShowPreview(true);
                    break;
                case "ArrowLeft":
                    if (document.activeElement?.tagName === "INPUT") return;
                    e.preventDefault();
                    setShowPreview(false);
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

            // Zoom shortcuts: Ctrl++ / Ctrl+- / Ctrl+0
            if (e.ctrlKey || e.metaKey) {
                if (e.key === "+" || e.key === "=") {
                    e.preventDefault();
                    zoomIn();
                } else if (e.key === "-") {
                    e.preventDefault();
                    zoomOut();
                } else if (e.key === "0") {
                    e.preventDefault();
                    zoomReset();
                }
            }
        };

        document.addEventListener("keydown", handleKeyDown, true); // capture phase
        return () => document.removeEventListener("keydown", handleKeyDown, true);
    }, [zoomIn, zoomOut, zoomReset]); // zoom callbacks are stable via useCallback

    // Clear search when switching tabs
    const handleTabChange = (tab) => {
        setActiveTab(tab);
        setSearchQuery("");
        if (tab !== "clipboard") setFilterType("all");
    };

    // Client-side filtering for clipboard items
    const filteredItems = items.filter((item) => {
        if (filterType === "all") return true;
        if (filterType === "pinned") return item.pinned;
        if (filterType === "text") return item.content_type === "text";
        if (filterType === "image") return item.content_type === "image";
        if (filterType === "files") return item.content_type === "files";
        return true;
    });

    // Determine search placeholder based on active tab
    const searchPlaceholder =
        activeTab === "emojis"
            ? t("search.placeholder_emojis")
            : activeTab === "symbols"
            ? t("search.placeholder_symbols")
            : activeTab === "gifs"
            ? t("search.placeholder_gifs")
            : t("search.placeholder_clipboard");

    return (
        <div className={`app${showPreview ? " app-split" : ""}`} role="application" aria-label={t("app.title")}>
            <div className="app-container">
                <AppHeader
                    onOpenSettings={() => setShowSettings(true)}
                    onClearAll={requestClearAll}
                    showPreview={showPreview}
                    onTogglePreview={() => setShowPreview((p) => !p)}
                />

                <TabBar activeTab={activeTab} onTabChange={handleTabChange} />

                <SearchBar
                    value={searchQuery}
                    onChange={setSearchQuery}
                    placeholder={searchPlaceholder}
                />

                <main role="main" aria-label={t("tabs." + activeTab)}>
                    <div className="main-split">
                        <div className="main-list">
                    {activeTab === "clipboard" && (
                        <>
                            <FilterPills activeFilter={filterType} onFilterChange={setFilterType} />
                            <ClipboardList
                                items={filteredItems}
                                selectedIndex={selectedIndex}
                                onPaste={handlePaste}
                                onPin={handlePin}
                                onDelete={handleDelete}
                                onLoadMore={handleLoadMore}
                                loading={loading}
                                hasMore={items.length < total}
                                onContextMenu={handleContextMenu}
                            />
                        </>
                    )}
                    {activeTab === "emojis" && (
                        <EmojiPicker searchQuery={searchQuery} onToast={showToast} />
                    )}
                    {activeTab === "symbols" && (
                        <SymbolPicker searchQuery={searchQuery} onToast={showToast} />
                    )}
                    {activeTab === "gifs" && (
                        <GifPicker searchQuery={searchQuery} onToast={showToast} />
                    )}
                        </div>
                        {showPreview && activeTab === "clipboard" && (
                            <PreviewPane
                                item={previewItem}
                                onPaste={handlePaste}
                                onToast={showToast}
                            />
                        )}
                    </div>
                </main>

                <Footer />
            </div>

            {toast && <div className="copy-toast" role="alert">{toast}</div>}

            {showSettings && (
                <SettingsPanel
                    onClose={() => setShowSettings(false)}
                    zoom={zoom}
                    onZoomChange={setZoom}
                />
            )}

            {showConfirm && (
                <ConfirmDialog
                    message={t("confirm.clear_all")}
                    onConfirm={handleConfirmClear}
                    onCancel={() => setShowConfirm(false)}
                />
            )}

            {ctxMenu && (
                <ContextMenu
                    item={ctxMenu.item}
                    x={ctxMenu.x}
                    y={ctxMenu.y}
                    onClose={() => setCtxMenu(null)}
                    onPin={handlePin}
                    onDelete={handleDelete}
                    onPaste={handlePaste}
                    onToast={showToast}
                    onShowQr={(text) => setQrText(text)}
                    onItemUpdate={handleItemUpdate}
                />
            )}

            {qrText && (
                <QrModal
                    text={qrText}
                    onClose={() => setQrText(null)}
                    onToast={showToast}
                />
            )}
        </div>
    );
}

export default App;
