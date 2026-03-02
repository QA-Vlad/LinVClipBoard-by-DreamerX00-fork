import { useEffect, useRef } from "react";

function ConfirmDialog({ message, onConfirm, onCancel }) {
    const cancelRef = useRef(null);

    // Auto-focus the Cancel button so accidental Enter won't confirm
    useEffect(() => {
        cancelRef.current?.focus();
    }, []);

    // Close on Escape
    useEffect(() => {
        const handler = (e) => {
            if (e.key === "Escape") {
                e.stopPropagation();
                onCancel();
            }
        };
        document.addEventListener("keydown", handler, true);
        return () => document.removeEventListener("keydown", handler, true);
    }, [onCancel]);

    return (
        <div
            className="confirm-overlay"
            role="alertdialog"
            aria-modal="true"
            aria-label="Confirmation"
            aria-describedby="confirm-msg"
        >
            <div className="confirm-dialog">
                <p id="confirm-msg" className="confirm-message">
                    {message}
                </p>
                <div className="confirm-actions">
                    <button ref={cancelRef} className="confirm-cancel" onClick={onCancel}>
                        Cancel
                    </button>
                    <button className="confirm-ok" onClick={onConfirm}>
                        Confirm
                    </button>
                </div>
            </div>
        </div>
    );
}

export default ConfirmDialog;
