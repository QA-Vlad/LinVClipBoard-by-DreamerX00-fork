mod dbus_service;
mod monitor;
mod server;

use shared::config::AppConfig;
use shared::db::Database;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config first so we can use its log_level
    let config = AppConfig::load();

    // Initialize logging with config log_level
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&config.daemon.log_level)),
        )
        .with_target(false)
        .compact()
        .init();

    tracing::info!("🚀 LinVClipBoard daemon v{} starting...", env!("CARGO_PKG_VERSION"));
    tracing::info!(
        "Config loaded (poll: {}ms, max_items: {})",
        config.daemon.poll_interval_ms,
        config.storage.max_items
    );

    // Ensure data directories exist
    let blob_dir = AppConfig::blob_dir();
    std::fs::create_dir_all(&blob_dir)?;

    // Open database
    let db_path = AppConfig::db_path();
    let db = Database::open(&db_path)?;
    tracing::info!("Database opened at {:?}", db_path);

    let db = Arc::new(db);
    let config = Arc::new(config);
    let start_time = std::time::Instant::now();
    let cancel = CancellationToken::new();

    // ── Clipboard monitor ────────────────────────────────────────────────
    let monitor_db = Arc::clone(&db);
    let monitor_config = Arc::clone(&config);
    let monitor_cancel = cancel.clone();
    let monitor_handle = tokio::spawn(async move {
        if let Err(e) = monitor::run(monitor_db, monitor_config, monitor_cancel).await {
            tracing::error!("Monitor error: {}", e);
        }
    });

    // ── IPC server ───────────────────────────────────────────────────────
    let server_db = Arc::clone(&db);
    let server_config = Arc::clone(&config);
    let socket_path = AppConfig::socket_path();
    let server_cancel = cancel.clone();
    let server_handle = tokio::spawn(async move {
        if let Err(e) =
            server::run(server_db, server_config, &socket_path, start_time, server_cancel).await
        {
            tracing::error!("Server error: {}", e);
        }
    });

    // ── D-Bus service (optional, feature-gated) ──────────────────────────
    let dbus_db = Arc::clone(&db);
    let _dbus_handle = tokio::spawn(async move {
        if let Err(e) = dbus_service::start_dbus_service(dbus_db).await {
            tracing::warn!("D-Bus service unavailable: {}", e);
        }
    });

    // ── Config file watcher ──────────────────────────────────────────────
    let config_path = AppConfig::config_path();
    let _config_watcher_handle = tokio::spawn(async move {
        use notify::{Event, EventKind, RecursiveMode, Watcher};
        let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(4);

        let _watcher = {
            let tx = tx.clone();
            let mut w = notify::recommended_watcher(move |res: Result<Event, _>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        let _ = tx.blocking_send(());
                    }
                }
            });
            match &mut w {
                Ok(watcher) => {
                    if let Err(e) = watcher.watch(&config_path, RecursiveMode::NonRecursive) {
                        tracing::warn!("Could not watch config file: {}", e);
                    }
                }
                Err(e) => tracing::warn!("Config watcher init failed: {}", e),
            }
            w
        };

        while let Some(()) = rx.recv().await {
            tracing::info!("Config file changed — reload will take effect on next daemon restart");
            // NOTE: Full hot-reload would require replacing the Arc<AppConfig>
            // atomically and re-registering the global shortcut. For safety,
            // we log the change; live reload is planned for a future release.
        }
    });

    tracing::info!(
        "✨ Daemon ready. Listening on {:?}",
        AppConfig::socket_path()
    );

    // ── Wait for shutdown signal (SIGINT **or** SIGTERM) ─────────────────
    let mut sigterm =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("SIGINT received, shutting down…");
        }
        _ = sigterm.recv() => {
            tracing::info!("SIGTERM received, shutting down…");
        }
    }

    // Signal all tasks to stop gracefully
    cancel.cancel();

    // Give tasks a moment to finish in-flight work
    let _ = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        let _ = monitor_handle.await;
        let _ = server_handle.await;
    })
    .await;

    // Clean up socket file
    let socket_path = AppConfig::socket_path();
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    tracing::info!("👋 Daemon stopped.");
    Ok(())
}
