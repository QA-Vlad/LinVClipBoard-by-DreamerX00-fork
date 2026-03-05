import React from "react";

/**
 * ErrorBoundary — catches unhandled React errors and displays a
 * friendly fallback UI instead of a blank white screen.
 *
 * Must be a class component (React requirement for error boundaries).
 */
class ErrorBoundary extends React.Component {
    constructor(props) {
        super(props);
        this.state = { hasError: false, error: null };
    }

    static getDerivedStateFromError(error) {
        return { hasError: true, error };
    }

    componentDidCatch(error, errorInfo) {
        console.error("[ErrorBoundary] Caught:", error, errorInfo);
    }

    handleReload = () => {
        window.location.reload();
    };

    render() {
        if (this.state.hasError) {
            return (
                <div style={styles.container}>
                    <div style={styles.card}>
                        <span style={styles.icon}>⚠️</span>
                        <h2 style={styles.title}>Something went wrong</h2>
                        <p style={styles.message}>
                            {this.state.error?.message || "An unexpected error occurred."}
                        </p>
                        <button style={styles.button} onClick={this.handleReload}>
                            🔄 Reload App
                        </button>
                    </div>
                </div>
            );
        }

        return this.props.children;
    }
}

const styles = {
    container: {
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        height: "100%",
        width: "100%",
        background: "rgba(15, 15, 25, 0.98)",
        fontFamily: "'Inter Variable', 'Inter', -apple-system, sans-serif",
    },
    card: {
        textAlign: "center",
        padding: "32px",
        borderRadius: "14px",
        background: "rgba(35, 35, 55, 0.92)",
        border: "1px solid rgba(148, 163, 184, 0.12)",
        maxWidth: "360px",
    },
    icon: {
        fontSize: "40px",
        display: "block",
        marginBottom: "12px",
    },
    title: {
        color: "#f1f5f9",
        fontSize: "18px",
        fontWeight: 600,
        margin: "0 0 8px",
    },
    message: {
        color: "#94a3b8",
        fontSize: "13px",
        margin: "0 0 20px",
        lineHeight: 1.5,
        wordBreak: "break-word",
    },
    button: {
        background: "#6366f1",
        color: "#fff",
        border: "none",
        borderRadius: "8px",
        padding: "10px 24px",
        fontSize: "14px",
        fontWeight: 500,
        cursor: "pointer",
    },
};

export default ErrorBoundary;
