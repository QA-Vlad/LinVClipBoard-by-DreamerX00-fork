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
        <div className="search-bar">
            <span className="search-icon">🔍</span>
            <input
                ref={inputRef}
                type="text"
                className="search-input"
                placeholder="Search clipboard history..."
                value={value}
                onChange={(e) => onChange(e.target.value)}
                autoComplete="off"
                spellCheck="false"
            />
            {value && (
                <button className="search-clear" onClick={() => onChange("")}>
                    ✕
                </button>
            )}
        </div>
    );
}

export default SearchBar;
