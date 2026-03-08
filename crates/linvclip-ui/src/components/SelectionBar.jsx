import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "../i18n/index.jsx";

function SelectionBar({ selectedIds, items, onClearSelection, onItemsChanged, onToast }) {
    const { t } = useTranslation();
    const count = selectedIds.size;

    const handleBulkPin = async (pinned) => {
        try {
            const ids = [...selectedIds];
            await invoke("bulk_pin", { ids, pinned });
            onItemsChanged();
            onClearSelection();
            onToast(pinned ? t("selection.pinned") : t("selection.unpinned"));
        } catch (err) {
            console.error("Bulk pin failed:", err);
        }
    };

    const handleBulkDelete = async () => {
        if (!confirm(t("confirm.bulk_delete"))) return;
        try {
            const ids = [...selectedIds];
            await invoke("bulk_delete", { ids });
            onItemsChanged();
            onClearSelection();
            onToast(t("selection.deleted"));
        } catch (err) {
            console.error("Bulk delete failed:", err);
        }
    };

    const handleMerge = async () => {
        // Merge selected text items into one clipboard entry
        const selected = items.filter((it) => selectedIds.has(it.id));
        const textItems = selected.filter(
            (it) => it.content_type === "plain_text" || it.content_type === "html" || it.content_type === "uri"
        );
        if (textItems.length < 2) {
            onToast(t("selection.merge_need_two"));
            return;
        }
        const merged = textItems.map((it) => it.preview_text || it.content).join("\n");
        try {
            await invoke("paste_raw_text", { text: merged });
            onClearSelection();
            onToast(t("selection.merged"));
        } catch (err) {
            console.error("Merge failed:", err);
        }
    };

    if (count === 0) return null;

    return (
        <div className="selection-bar">
            <span className="selection-count">
                {t("selection.count").replace("{n}", count)}
            </span>
            <div className="selection-actions">
                <button className="sel-btn sel-btn-pin" onClick={() => handleBulkPin(true)} title={t("selection.pin_all")}>
                    📌 {t("selection.pin_all")}
                </button>
                <button className="sel-btn sel-btn-unpin" onClick={() => handleBulkPin(false)} title={t("selection.unpin_all")}>
                    📍 {t("selection.unpin_all")}
                </button>
                <button className="sel-btn sel-btn-merge" onClick={handleMerge} title={t("selection.merge")}>
                    🔗 {t("selection.merge")}
                </button>
                <button className="sel-btn sel-btn-delete" onClick={handleBulkDelete} title={t("selection.delete_all")}>
                    🗑️ {t("selection.delete_all")}
                </button>
                <button className="sel-btn sel-btn-clear" onClick={onClearSelection} title={t("selection.clear")}>
                    ✕ {t("selection.clear")}
                </button>
            </div>
        </div>
    );
}

export default SelectionBar;
