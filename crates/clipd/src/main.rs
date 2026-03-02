mod monitor;
mod server;

use shared::config::AppConfig;
use shared::db::Database;
use std::sync::Arc;
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

    tracing::info!("🚀 LinVClipBoard daemon starting...");
    tracing::info!("Config loaded (poll: {}ms, max_items: {})",
        config.daemon.poll_interval_ms, config.storage.max_items);

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

    // Spawn clipboard monitor
    let monitor_db = Arc::clone(&db);
    let monitor_config = Arc::clone(&config);
    let monitor_handle = tokio::spawn(async move {
        if let Err(e) = monitor::run(monitor_db, monitor_config).await {
            tracing::error!("Monitor error: {}", e);
        }
    });

    // Spawn IPC server
    let server_db = Arc::clone(&db);
    let server_config = Arc::clone(&config);
    let socket_path = AppConfig::socket_path();
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server::run(server_db, server_config, &socket_path, start_time).await {
            tracing::error!("Server error: {}", e);
        }
    });

    tracing::info!("✨ Daemon ready. Listening on {:?}", AppConfig::socket_path());

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutdown signal received, cleaning up...");

    // Clean up socket file
    let socket_path = AppConfig::socket_path();
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    monitor_handle.abort();
    server_handle.abort();

    tracing::info!("👋 Daemon stopped.");
    Ok(())
}
