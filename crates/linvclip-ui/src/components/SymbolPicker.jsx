import { useState, useMemo, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "../i18n/index.jsx";
import symbolData from "../data/symbols.json";

const CATEGORY_KEYS = {
    "Math": "symbol.math",
    "Arrows": "symbol.arrows",
    "Currency": "symbol.currency",
    "Greek": "symbol.greek",
    "Superscript & Subscript": "symbol.superscript",
    "Punctuation": "symbol.punctuation",
    "Box Drawing": "symbol.box_drawing",
};

const RECENT_KEY = "linvclip_recent_symbols";
const MAX_RECENT = 24;

function getRecent() {
    try {
        return JSON.parse(localStorage.getItem(RECENT_KEY) || "[]");
    } catch {
        return [];
    }
}

function addRecent(sym) {
    const arr = getRecent().filter((s) => s !== sym);
    arr.unshift(sym);
    localStorage.setItem(RECENT_KEY, JSON.stringify(arr.slice(0, MAX_RECENT)));
}

function SymbolPicker({ searchQuery, onToast }) {
    const { t } = useTranslation();
    const [recent, setRecent] = useState(getRecent);

    const handleClick = useCallback(async (sym) => {
        try {
            const result = await invoke("type_text", { text: sym });
            addRecent(sym);
            setRecent(getRecent());
            if (result === "typed") {
                if (onToast) onToast("✅ " + t("clipboard.inserted"));
            } else {
                if (onToast) onToast("📋 " + t("clipboard.copied"));
            }
        } catch (err) {
            console.error("Failed to insert symbol:", err);
            if (onToast) onToast("❌ " + t("clipboard.copy_failed"));
        }
    }, [t, onToast]);

    const query = (searchQuery || "").toLowerCase().trim();

    const filteredData = useMemo(() => {
        if (!query) return null;
        const results = [];
        for (const [category, symbols] of Object.entries(symbolData)) {
            for (const item of symbols) {
                if (item.name.includes(query) || item.symbol.includes(query)) {
                    results.push(item);
                }
            }
        }
        return results;
    }, [query]);

    if (filteredData !== null) {
        if (filteredData.length === 0) {
            return (
                <div className="picker-empty">
                    <p>{t("symbol.no_results")}</p>
                </div>
            );
        }
        return (
            <div className="picker-scroll">
                <div className="symbol-grid">
                    {filteredData.map((item, i) => (
                        <button
                            key={i}
                            className="symbol-cell"
                            onClick={() => handleClick(item.symbol)}
                            title={item.name}
                            aria-label={item.name}
                        >
                            {item.symbol}
                        </button>
                    ))}
                </div>
            </div>
        );
    }

    return (
        <div className="picker-scroll">
            {recent.length > 0 && (
                <>
                    <h3 className="picker-category-header">{t("symbol.recent")}</h3>
                    <div className="symbol-grid">
                        {recent.map((sym, i) => (
                            <button
                                key={"r" + i}
                                className="symbol-cell"
                                onClick={() => handleClick(sym)}
                                title={sym}
                                aria-label={sym}
                            >
                                {sym}
                            </button>
                        ))}
                    </div>
                </>
            )}

            {Object.entries(symbolData).map(([category, symbols]) => (
                <div key={category}>
                    <h3 className="picker-category-header">
                        {t(CATEGORY_KEYS[category] || category)}
                    </h3>
                    <div className="symbol-grid">
                        {symbols.map((item, i) => (
                            <button
                                key={i}
                                className="symbol-cell"
                                onClick={() => handleClick(item.symbol)}
                                title={item.name}
                                aria-label={item.name}
                            >
                                {item.symbol}
                            </button>
                        ))}
                    </div>
                </div>
            ))}
        </div>
    );
}

export default SymbolPicker;
