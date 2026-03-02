import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

const ZOOM_MIN = 50;
const ZOOM_MAX = 200;
const ZOOM_STEP = 10;
const ZOOM_DEFAULT = 100;

/**
 * Custom hook for managing UI zoom.
 * Reads initial zoom from backend config, applies it via CSS custom property,
 * and exposes setZoom for live updates.
 */
export function useZoom() {
    const [zoom, setZoomState] = useState(() => {
        const stored = localStorage.getItem("linvclip_zoom");
        return stored ? parseInt(stored, 10) : ZOOM_DEFAULT;
    });

    // Apply zoom to root element
    useEffect(() => {
        const factor = zoom / 100;
        document.documentElement.style.fontSize = `${factor * 16}px`;
        document.documentElement.style.setProperty("--zoom-factor", String(factor));
        localStorage.setItem("linvclip_zoom", String(zoom));
    }, [zoom]);

    // Load zoom from backend config on mount
    useEffect(() => {
        (async () => {
            try {
                const cfg = await invoke("get_config");
                if (cfg.ui?.zoom && cfg.ui.zoom !== zoom) {
                    setZoomState(cfg.ui.zoom);
                }
            } catch (_) {
                // Fallback to localStorage value
            }
        })();
    }, []);

    const setZoom = useCallback((val) => {
        const clamped = Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, parseInt(val, 10) || ZOOM_DEFAULT));
        setZoomState(clamped);
    }, []);

    const zoomIn = useCallback(() => {
        setZoomState((prev) => Math.min(ZOOM_MAX, prev + ZOOM_STEP));
    }, []);

    const zoomOut = useCallback(() => {
        setZoomState((prev) => Math.max(ZOOM_MIN, prev - ZOOM_STEP));
    }, []);

    const zoomReset = useCallback(() => {
        setZoomState(ZOOM_DEFAULT);
    }, []);

    return { zoom, setZoom, zoomIn, zoomOut, zoomReset };
}
