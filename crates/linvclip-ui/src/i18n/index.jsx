import { createContext, useContext, useState, useCallback, useEffect } from "react";
import en from "./en.json";
import pt from "./pt.json";
import ja from "./japanese.json";
import hi from "./hin.json";

const locales = { en, pt, ja, hi };
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
 * @param {string} defaultLang — initial language code ("en" | "pt" | "ja" | "hi    ")
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
    (key) => {
        const current = resolve(locales[lang], key);
        if (current !== key) return current;

        // fallback to English
        const fallback = resolve(locales.en, key);
        return fallback !== key ? fallback : key;
    },
    [lang]
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
