import { useRef, useEffect } from "react";

function SearchBar({ value, onChange }) {
    const inputRef = useRef(null);

    // Auto-focus on mount
    useEffect(() => {
        if (inputRef.current) {
            inputRef.current.focus();
        }
    }, []);

    return (
        <div className="search-bar" role="search" aria-label="Search clipboard">
            <span className="search-icon" aria-hidden="true">🔍</span>
            <input
                ref={inputRef}
                type="text"
                className="search-input"
                placeholder="Search clipboard history..."
                value={value}
                onChange={(e) => onChange(e.target.value)}
                autoComplete="off"
                spellCheck="false"
                aria-label="Search clipboard history"
                role="searchbox"
            />
            {value && (
                <button
                    className="search-clear"
                    onClick={() => onChange("")}
                    aria-label="Clear search"
                >
                    ✕
                </button>
            )}
        </div>
    );
}

export default SearchBar;
