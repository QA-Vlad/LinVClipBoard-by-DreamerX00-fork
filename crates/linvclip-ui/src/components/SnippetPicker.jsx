import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "../i18n/index.jsx";
import SnippetEditor from "./SnippetEditor.jsx";
import SnippetVarDialog from "./SnippetVarDialog.jsx";

function SnippetPicker({ searchQuery, onToast, initialContent, onConsumeInitContent }) {
    const { t } = useTranslation();
    const [snippets, setSnippets] = useState([]);
    const [folders, setFolders] = useState([]);
    const [activeFolder, setActiveFolder] = useState(null); // null = All
    const [editorOpen, setEditorOpen] = useState(false);
    const [editingSnippet, setEditingSnippet] = useState(null);
    const [varDialogSnippet, setVarDialogSnippet] = useState(null);

    // Auto-open editor when initialContent is provided (from "Save as Snippet")
    useEffect(() => {
        if (initialContent) {
            setEditingSnippet({ content: initialContent, name: "", folder: "", abbreviation: "", variables: "[]" });
            setEditorOpen(true);
            if (onConsumeInitContent) onConsumeInitContent();
        }
    }, [initialContent]);

    const fetchSnippets = useCallback(async () => {
        try {
            let result;
            if (searchQuery && searchQuery.trim()) {
                result = await invoke("search_snippets", { query: searchQuery });
            } else {
                result = await invoke("list_snippets", {
                    folder: activeFolder || null,
                });
            }
            setSnippets(result);

            // Extract unique folders
            const allSnippets = await invoke("list_snippets", { folder: null });
            const uniqueFolders = [
                ...new Set(allSnippets.map((s) => s.folder).filter(Boolean)),
            ].sort();
            setFolders(uniqueFolders);
        } catch (err) {
            console.error("Failed to fetch snippets:", err);
        }
    }, [searchQuery, activeFolder]);

    useEffect(() => {
        fetchSnippets();
    }, [fetchSnippets]);

    const handleUse = async (snippet) => {
        // Parse variables from content
        const varMatches = [...snippet.content.matchAll(/\{\{(\w+)\}\}/g)];
        const varNames = [...new Set(varMatches.map((m) => m[1]))];

        if (varNames.length === 0) {
            // No variables — use immediately
            try {
                await invoke("use_snippet", { id: snippet.id, variables: {} });
                onToast(t("snippets.copied"));
                fetchSnippets(); // refresh use_count
            } catch (err) {
                console.error("Use snippet failed:", err);
            }
        } else {
            // Has variables — show dialog
            setVarDialogSnippet(snippet);
        }
    };

    const handleVarSubmit = async (variables) => {
        if (!varDialogSnippet) return;
        try {
            await invoke("use_snippet", {
                id: varDialogSnippet.id,
                variables,
            });
            onToast(t("snippets.copied"));
            setVarDialogSnippet(null);
            fetchSnippets();
        } catch (err) {
            console.error("Use snippet failed:", err);
        }
    };

    const handleEdit = (snippet) => {
        setEditingSnippet(snippet);
        setEditorOpen(true);
    };

    const handleDelete = async (snippet) => {
        if (!confirm(t("snippets.confirm_delete").replace("{name}", snippet.name)))
            return;
        try {
            await invoke("delete_snippet", { id: snippet.id });
            fetchSnippets();
            onToast(t("snippets.deleted"));
        } catch (err) {
            console.error("Delete snippet failed:", err);
        }
    };

    const handleSave = () => {
        setEditorOpen(false);
        setEditingSnippet(null);
        fetchSnippets();
    };

    if (editorOpen) {
        return (
            <SnippetEditor
                snippet={editingSnippet}
                folders={folders}
                onSave={handleSave}
                onCancel={() => {
                    setEditorOpen(false);
                    setEditingSnippet(null);
                }}
                onToast={onToast}
            />
        );
    }

    return (
        <div className="snippet-picker">
            {varDialogSnippet && (
                <SnippetVarDialog
                    snippet={varDialogSnippet}
                    onSubmit={handleVarSubmit}
                    onCancel={() => setVarDialogSnippet(null)}
                />
            )}

            <div className="snippet-layout">
                {/* Folder sidebar */}
                <div className="snippet-folders">
                    <button
                        className={`folder-btn${activeFolder === null ? " folder-active" : ""}`}
                        onClick={() => setActiveFolder(null)}
                    >
                        {t("filters.all")}
                    </button>
                    {folders.map((f) => (
                        <button
                            key={f}
                            className={`folder-btn${activeFolder === f ? " folder-active" : ""}`}
                            onClick={() => setActiveFolder(f)}
                        >
                            {f}
                        </button>
                    ))}
                </div>

                {/* Snippet list */}
                <div className="snippet-list">
                    <div className="snippet-header">
                        <button
                            className="snippet-new-btn"
                            onClick={() => {
                                setEditingSnippet(null);
                                setEditorOpen(true);
                            }}
                        >
                            + {t("snippets.new")}
                        </button>
                    </div>

                    {snippets.length === 0 ? (
                        <div className="empty-state">
                            <span className="empty-icon">📝</span>
                            <p className="empty-title">{t("snippets.no_snippets")}</p>
                        </div>
                    ) : (
                        snippets.map((snippet) => (
                            <div
                                key={snippet.id}
                                className="snippet-item"
                                onClick={() => handleUse(snippet)}
                            >
                                <div className="snippet-item-header">
                                    <span className="snippet-name">{snippet.name}</span>
                                    {snippet.abbreviation && (
                                        <span className="snippet-abbr">
                                            {snippet.abbreviation}
                                        </span>
                                    )}
                                    {snippet.use_count > 0 && (
                                        <span className="snippet-use-count">
                                            x{snippet.use_count}
                                        </span>
                                    )}
                                </div>
                                <div className="snippet-preview">
                                    {snippet.content.slice(0, 80).replace(/\n/g, " ")}
                                    {snippet.content.length > 80 ? "..." : ""}
                                </div>
                                <div className="snippet-item-actions">
                                    <button
                                        className="action-btn"
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            handleEdit(snippet);
                                        }}
                                        title={t("snippets.edit")}
                                    >
                                        ✏️
                                    </button>
                                    <button
                                        className="action-btn delete-btn"
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            handleDelete(snippet);
                                        }}
                                        title={t("snippets.delete")}
                                    >
                                        🗑️
                                    </button>
                                </div>
                            </div>
                        ))
                    )}
                </div>
            </div>
        </div>
    );
}

export default SnippetPicker;
