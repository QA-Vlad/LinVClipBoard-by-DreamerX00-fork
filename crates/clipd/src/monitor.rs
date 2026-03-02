use arboard::Clipboard;
use sha2::{Digest, Sha256};
use shared::config::AppConfig;
use shared::db::Database;
use shared::models::{ClipboardItem, ContentType};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Run the clipboard monitor loop.
pub async fn run(
    db: Arc<Database>,
    config: Arc<AppConfig>,
    cancel: CancellationToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let base_poll = Duration::from_millis(config.daemon.poll_interval_ms);
    let mut last_text_checksum = String::new();
    let mut last_image_checksum = String::new();

    // Track limit enforcement — run every 100 polls
    let mut poll_count: u64 = 0;

    // Adaptive polling: speed up when activity detected, slow down when idle
    let mut idle_streak: u64 = 0;

    // Consecutive failure counter for clipboard reconnect
    let mut consecutive_failures: u32 = 0;
    let mut backoff_secs: u64 = 1;
    const MAX_BACKOFF_SECS: u64 = 60;
    const RECONNECT_THRESHOLD: u32 = 3;

    // Create clipboard instance ONCE — avoids Wayland fallback warning spam.
    let mut clipboard = Clipboard::new()?;
    tracing::info!(
        "📋 Clipboard monitor started (polling every {}ms)",
        config.daemon.poll_interval_ms
    );

    loop {
        // Check for cancellation before each poll
        if cancel.is_cancelled() {
            tracing::info!("Monitor received cancellation, stopping");
            break;
        }

        let mut had_activity = false;

        if !config.security.incognito {
            let mut had_failure = false;

            // Try to capture text
            match capture_text(&mut clipboard, &last_text_checksum) {
                Ok(Some((item, checksum))) => match db.insert(&item) {
                    Ok(true) => {
                        last_text_checksum = checksum;
                        tracing::debug!("Captured text: {} chars", item.size_bytes);
                        had_activity = true;
                    }
                    Ok(false) => {
                        last_text_checksum = checksum;
                        tracing::debug!("Duplicate text skipped");
                    }
                    Err(e) => tracing::error!("DB insert error: {}", e),
                },
                Ok(None) => {} // No change
                Err(_) => {
                    had_failure = true;
                }
            }

            // Try to capture image
            match capture_image(&mut clipboard, &last_image_checksum, &config) {
                Ok(Some((item, checksum))) => match db.insert(&item) {
                    Ok(true) => {
                        last_image_checksum = checksum;
                        tracing::debug!("Captured image: {} bytes", item.size_bytes);
                        had_activity = true;
                    }
                    Ok(false) => {
                        last_image_checksum = checksum;
                    }
                    Err(e) => tracing::error!("DB insert error: {}", e),
                },
                Ok(None) => {}
                Err(_) => {
                    had_failure = true;
                }
            }

            // Reconnect clipboard on consecutive failures
            if had_failure {
                consecutive_failures += 1;
                if consecutive_failures >= RECONNECT_THRESHOLD {
                    tracing::warn!(
                        "Clipboard failed {} times, reconnecting (backoff {}s)...",
                        consecutive_failures,
                        backoff_secs
                    );
                    tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                    match Clipboard::new() {
                        Ok(new_clip) => {
                            clipboard = new_clip;
                            consecutive_failures = 0;
                            backoff_secs = 1;
                            tracing::info!("Clipboard reconnected successfully");
                        }
                        Err(e) => {
                            tracing::error!("Clipboard reconnect failed: {}", e);
                            backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF_SECS);
                        }
                    }
                }
            } else {
                consecutive_failures = 0;
                backoff_secs = 1;
            }
        }

        // Enforce storage limits periodically
        poll_count += 1;
        if poll_count.is_multiple_of(100) {
            if let Err(e) = db.enforce_limits(&config.storage) {
                tracing::error!("Limit enforcement error: {}", e);
            }
            // Clean up orphan blobs
            let blob_dir = AppConfig::blob_dir();
            if let Err(e) = db.cleanup_orphan_blobs(&blob_dir) {
                tracing::error!("Orphan blob cleanup error: {}", e);
            }
        }

        // Adaptive polling: use base interval when active, gradually slow down when idle
        if had_activity {
            idle_streak = 0;
        } else {
            idle_streak += 1;
        }
        let poll_interval = if idle_streak > 40 {
            // After ~10 seconds of inactivity, slow to 2x base interval
            base_poll * 2
        } else {
            base_poll
        };

        tokio::select! {
            _ = tokio::time::sleep(poll_interval) => {}
            _ = cancel.cancelled() => {
                tracing::info!("Monitor cancelled during sleep");
                break;
            }
        }
    }

    Ok(())
}

/// Attempt to capture text from clipboard. Returns Some if content changed.
fn capture_text(
    clipboard: &mut Clipboard,
    last_checksum: &str,
) -> Result<Option<(ClipboardItem, String)>, Box<dyn std::error::Error + Send + Sync>> {
    let text = clipboard.get_text()?;

    if text.trim().is_empty() {
        return Ok(None);
    }

    // Normalize text to prevent encoding-caused duplicates
    let normalized = text.replace("\r\n", "\n").trim().to_string();

    // Compute checksum on normalized text
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let checksum = hex::encode(hasher.finalize());

    if checksum == last_checksum {
        return Ok(None);
    }

    let preview = normalized.chars().take(200).collect::<String>();
    let size = normalized.len() as u64;

    // Store the *normalized* text so content matches its checksum (#1)
    let item = ClipboardItem::new(
        ContentType::PlainText,
        normalized,
        preview,
        checksum.clone(),
        size,
    );

    Ok(Some((item, checksum)))
}

/// Attempt to capture an image from clipboard. Returns Some if content changed.
fn capture_image(
    clipboard: &mut Clipboard,
    last_checksum: &str,
    config: &AppConfig,
) -> Result<Option<(ClipboardItem, String)>, Box<dyn std::error::Error + Send + Sync>> {
    let img_data = clipboard.get_image()?;

    let raw_bytes = &img_data.bytes;
    if raw_bytes.is_empty() {
        return Ok(None);
    }

    // Compute checksum of raw image bytes
    let mut hasher = Sha256::new();
    hasher.update(raw_bytes.as_ref());
    let checksum = hex::encode(hasher.finalize());

    if checksum == last_checksum {
        return Ok(None);
    }

    // Check size limit
    let size = raw_bytes.len() as u64;
    if size > config.storage.max_item_size_bytes {
        tracing::warn!(
            "Image too large: {} bytes (limit: {})",
            size,
            config.storage.max_item_size_bytes
        );
        return Ok(None);
    }

    // Save as PNG file in blob directory
    let blob_dir = AppConfig::blob_dir();
    std::fs::create_dir_all(&blob_dir)?;
    let filename = format!("{}.png", Uuid::new_v4());
    let blob_path = blob_dir.join(&filename);

    // Create image buffer from raw RGBA data
    let img_buf = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
        img_data.width as u32,
        img_data.height as u32,
        raw_bytes.to_vec(),
    );

    if let Some(buf) = img_buf {
        buf.save(&blob_path)?;
    } else {
        // Fallback: save raw bytes
        std::fs::write(&blob_path, raw_bytes.as_ref())?;
    }

    let preview = format!("Image {}x{}", img_data.width, img_data.height);
    let content = blob_path.to_string_lossy().to_string();

    let item = ClipboardItem::new(ContentType::Image, content, preview, checksum.clone(), size);

    Ok(Some((item, checksum)))
}
