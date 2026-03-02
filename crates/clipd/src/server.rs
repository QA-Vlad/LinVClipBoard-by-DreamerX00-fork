use arboard::Clipboard;
use shared::config::AppConfig;
use shared::db::Database;
use shared::ipc::{recv_message, send_message};
use shared::models::{ContentType, IpcRequest, IpcResponse};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UnixListener;
use tokio::sync::Semaphore;

/// Run the IPC server on a Unix domain socket.
pub async fn run(
    db: Arc<Database>,
    _config: Arc<AppConfig>,
    socket_path: &Path,
    start_time: Instant,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Remove stale socket file
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    let semaphore = Arc::new(Semaphore::new(10));
    tracing::info!("🔌 IPC server listening on {:?}", socket_path);

    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                let db = Arc::clone(&db);
                let start = start_time;
                let sem = Arc::clone(&semaphore);

                tokio::spawn(async move {
                    let _permit = match sem.acquire().await {
                        Ok(p) => p,
                        Err(_) => return,
                    };

                    match recv_message::<IpcRequest>(&mut stream).await {
                        Ok(request) => {
                            let response = handle_request(&db, request, start);
                            if let Err(e) = send_message(&mut stream, &response).await {
                                tracing::error!("Failed to send response: {}", e);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to receive request: {}", e);
                            let _ = send_message(
                                &mut stream,
                                &IpcResponse::Error {
                                    message: format!("Invalid request: {}", e),
                                },
                            )
                            .await;
                        }
                    }
                });
            }
            Err(e) => {
                tracing::error!("Accept error: {}", e);
            }
        }
    }
}

/// Handle an IPC request and return a response.
fn handle_request(
    db: &Database,
    request: IpcRequest,
    start_time: Instant,
) -> IpcResponse {
    match request {
        IpcRequest::List { offset, limit } => {
            match db.list(offset, limit) {
                Ok((items, total)) => IpcResponse::Items { items, total },
                Err(e) => IpcResponse::Error {
                    message: format!("List failed: {}", e),
                },
            }
        }

        IpcRequest::Search { query, limit } => {
            match db.search(&query, limit) {
                Ok((items, total)) => IpcResponse::Items { items, total },
                Err(e) => IpcResponse::Error {
                    message: format!("Search failed: {}", e),
                },
            }
        }

        IpcRequest::Get { id } => {
            match db.get(&id) {
                Ok(item) => IpcResponse::Item(item),
                Err(e) => IpcResponse::Error {
                    message: format!("Get failed: {}", e),
                },
            }
        }

        IpcRequest::Delete { id } => {
            match db.delete(&id) {
                Ok(()) => IpcResponse::Ok {
                    message: format!("Deleted item {}", id),
                },
                Err(e) => IpcResponse::Error {
                    message: format!("Delete failed: {}", e),
                },
            }
        }

        IpcRequest::BulkDelete { ids } => {
            match db.bulk_delete(&ids) {
                Ok(count) => IpcResponse::Ok {
                    message: format!("Deleted {} items", count),
                },
                Err(e) => IpcResponse::Error {
                    message: format!("Bulk delete failed: {}", e),
                },
            }
        }

        IpcRequest::TogglePin { id } => {
            match db.toggle_pin(&id) {
                Ok(item) => IpcResponse::Item(item),
                Err(e) => IpcResponse::Error {
                    message: format!("Toggle pin failed: {}", e),
                },
            }
        }

        IpcRequest::Paste { id } => {
            match db.get(&id) {
                Ok(item) => {
                    match item.content_type {
                        ContentType::PlainText | ContentType::Html | ContentType::RichText => {
                            match Clipboard::new() {
                                Ok(mut clip) => {
                                    match clip.set_text(&item.content) {
                                        Ok(()) => IpcResponse::Ok {
                                            message: "Pasted to clipboard".to_string(),
                                        },
                                        Err(e) => IpcResponse::Error {
                                            message: format!("Clipboard set failed: {}", e),
                                        },
                                    }
                                }
                                Err(e) => IpcResponse::Error {
                                    message: format!("Clipboard init failed: {}", e),
                                },
                            }
                        }
                        ContentType::Image => {
                            // Read image file and set to clipboard
                            let img_path = std::path::Path::new(&item.content);
                            if img_path.exists() {
                                match image::open(img_path) {
                                    Ok(img) => {
                                        let rgba = img.to_rgba8();
                                        let (w, h) = rgba.dimensions();
                                        let img_data = arboard::ImageData {
                                            width: w as usize,
                                            height: h as usize,
                                            bytes: std::borrow::Cow::Owned(rgba.into_raw()),
                                        };
                                        match Clipboard::new() {
                                            Ok(mut clip) => {
                                                match clip.set_image(img_data) {
                                                    Ok(()) => IpcResponse::Ok {
                                                        message: "Image pasted to clipboard".to_string(),
                                                    },
                                                    Err(e) => IpcResponse::Error {
                                                        message: format!("Image paste failed: {}", e),
                                                    },
                                                }
                                            }
                                            Err(e) => IpcResponse::Error {
                                                message: format!("Clipboard init failed: {}", e),
                                            },
                                        }
                                    }
                                    Err(e) => IpcResponse::Error {
                                        message: format!("Image read failed: {}", e),
                                    },
                                }
                            } else {
                                IpcResponse::Error {
                                    message: format!("Image file not found: {}", item.content),
                                }
                            }
                        }
                    }
                }
                Err(e) => IpcResponse::Error {
                    message: format!("Item not found: {}", e),
                },
            }
        }

        IpcRequest::Clear => {
            match db.clear_unpinned() {
                Ok(count) => IpcResponse::Ok {
                    message: format!("Cleared {} items (pinned items kept)", count),
                },
                Err(e) => IpcResponse::Error {
                    message: format!("Clear failed: {}", e),
                },
            }
        }

        IpcRequest::Status => {
            let uptime = start_time.elapsed().as_secs();
            let total_items = db.total_items().unwrap_or(0);
            let db_size = db.db_size().unwrap_or(0);

            IpcResponse::Status {
                uptime_secs: uptime,
                total_items,
                db_size_bytes: db_size,
            }
        }
    }
}
