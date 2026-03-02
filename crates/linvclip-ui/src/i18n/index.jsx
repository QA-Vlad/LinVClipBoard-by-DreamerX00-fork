import { createContext, useContext, useState, useCallback, useEffect } from "react";
import en from "./en.json";
import pt from "./pt.json";

const locales = { en, pt };
const I18nContext = createContext(null);

/**
 * Resolve a dot-separated key path (e.g. "settings.title") from a locale object.
 */
function resolve(obj, path) {
    return path.split(".").reduce((acc, key) => (acc && acc[key] !== undefined ? acc[key] : path), obj);
}

/**
 * I18nProvider — wrap your app to provide translations.
 *
 * @param {string} defaultLang — initial language code ("en" | "pt")
 */
export function I18nProvider({ defaultLang = "en", children }) {
    const [lang, setLang] = useState(() => {
        // Try persisted value first
        const stored = localStorage.getItem("language");
        if (stored && locales[stored]) return stored;
        return defaultLang;
    });

    // Persist language choice
    useEffect(() => {
        localStorage.setItem("language", lang);
        document.documentElement.setAttribute("lang", lang);
    }, [lang]);

    /** Translate a key path. Returns the key itself if not found. */
    const t = useCallback(
        (key) => resolve(locales[lang] || locales.en, key),
        [lang],
    );

    /** List of available language codes. */
    const availableLanguages = Object.keys(locales);

    return (
        <I18nContext.Provider value={{ lang, setLang, t, availableLanguages }}>
            {children}
        </I18nContext.Provider>
    );
}

/**
 * Hook to access the translation function and language control.
 *
 * @returns {{ t: (key: string) => string, lang: string, setLang: (code: string) => void, availableLanguages: string[] }}
 */
export function useTranslation() {
    const ctx = useContext(I18nContext);
    if (!ctx) throw new Error("useTranslation must be used within <I18nProvider>");
    return ctx;
}
