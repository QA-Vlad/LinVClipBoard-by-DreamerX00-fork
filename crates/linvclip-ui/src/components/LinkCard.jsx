import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

/**
 * Displays an Open-Graph link preview card for a URL.
 * Fetches title, description, image, site name, and favicon from the backend.
 */
function LinkCard({ url }) {
    const [data, setData] = useState(null);
    const [error, setError] = useState(false);

    useEffect(() => {
        let cancelled = false;
        setData(null);
        setError(false);

        invoke("fetch_link_preview", { url })
            .then((d) => { if (!cancelled) setData(d); })
            .catch(() => { if (!cancelled) setError(true); });

        return () => { cancelled = true; };
    }, [url]);

    if (error) {
        return (
            <div className="link-card link-card-error">
                <a href={url} target="_blank" rel="noopener noreferrer" className="link-card-url">{url}</a>
            </div>
        );
    }

    if (!data) {
        return (
            <div className="link-card link-card-loading">
                <div className="spinner" />
                <span className="link-card-url-small">{url}</span>
            </div>
        );
    }

    return (
        <div className="link-card">
            {data.image && (
                <div className="link-card-image">
                    <img src={data.image} alt="" loading="lazy" />
                </div>
            )}
            <div className="link-card-body">
                <div className="link-card-header">
                    {data.favicon && <img className="link-card-favicon" src={data.favicon} alt="" />}
                    {data.site_name && <span className="link-card-site">{data.site_name}</span>}
                </div>
                {data.title && <h3 className="link-card-title">{data.title}</h3>}
                {data.description && <p className="link-card-desc">{data.description}</p>}
                <a href={url} target="_blank" rel="noopener noreferrer" className="link-card-url-small">{url}</a>
            </div>
        </div>
    );
}

export default LinkCard;
