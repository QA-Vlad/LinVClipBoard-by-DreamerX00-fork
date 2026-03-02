use base64::Engine;
use serde::{Deserialize, Serialize};
use shared::config::AppConfig;
use shared::ipc::send_request;
use shared::models::{ClipboardItem, IpcRequest, IpcResponse};
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

#[derive(Serialize, Deserialize)]
pub struct ItemsResult {
    pub items: Vec<ClipboardItem>,
    pub total: u64,
}

#[derive(Serialize, Deserialize)]
pub struct StatusResult {
    pub uptime_secs: u64,
    pub total_items: u64,
    pub db_size_bytes: u64,
}

/// Get clipboard items from the daemon.
#[tauri::command]
async fn get_items(offset: u32, limit: u32) -> Result<ItemsResult, String> {
    let socket = AppConfig::socket_path();
    let request = IpcRequest::List { offset, limit };

    match send_request(&socket, &request).await {
        Ok(IpcResponse::Items { items, total }) => Ok(ItemsResult { items, total }),
        Ok(IpcResponse::Error { message }) => Err(message),
        Err(e) => Err(format!("Connection failed: {}. Is clipd running?", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

/// Search clipboard items.
#[tauri::command]
async fn search_items(query: String, limit: u32) -> Result<ItemsResult, String> {
    let socket = AppConfig::socket_path();
    let request = IpcRequest::Search { query, limit };

    match send_request(&socket, &request).await {
        Ok(IpcResponse::Items { items, total }) => Ok(ItemsResult { items, total }),
        Ok(IpcResponse::Error { message }) => Err(message),
        Err(e) => Err(format!("Connection failed: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

/// Paste an item back to clipboard.
#[tauri::command]
async fn paste_item(id: String) -> Result<String, String> {
    let socket = AppConfig::socket_path();
    let request = IpcRequest::Paste { id };

    match send_request(&socket, &request).await {
        Ok(IpcResponse::Ok { message }) => Ok(message),
        Ok(IpcResponse::Error { message }) => Err(message),
        Err(e) => Err(format!("Connection failed: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

/// Write arbitrary text (emoji/symbol) directly to the system clipboard.
#[tauri::command]
async fn paste_raw_text(text: String) -> Result<String, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| format!("Clipboard error: {}", e))?;
    clipboard
        .set_text(&text)
        .map_err(|e| format!("Failed to set clipboard: {}", e))?;
    Ok("ok".to_string())
}

/// Toggle pin on an item.
#[tauri::command]
async fn pin_item(id: String) -> Result<ClipboardItem, String> {
    let socket = AppConfig::socket_path();
    let request = IpcRequest::TogglePin { id };

    match send_request(&socket, &request).await {
        Ok(IpcResponse::Item(item)) => Ok(item),
        Ok(IpcResponse::Error { message }) => Err(message),
        Err(e) => Err(format!("Connection failed: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

/// Delete an item.
#[tauri::command]
async fn delete_item(id: String) -> Result<String, String> {
    let socket = AppConfig::socket_path();
    let request = IpcRequest::Delete { id };

    match send_request(&socket, &request).await {
        Ok(IpcResponse::Ok { message }) => Ok(message),
        Ok(IpcResponse::Error { message }) => Err(message),
        Err(e) => Err(format!("Connection failed: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

/// Get daemon status.
#[tauri::command]
async fn get_status() -> Result<StatusResult, String> {
    let socket = AppConfig::socket_path();
    let request = IpcRequest::Status;

    match send_request(&socket, &request).await {
        Ok(IpcResponse::Status {
            uptime_secs,
            total_items,
            db_size_bytes,
        }) => Ok(StatusResult {
            uptime_secs,
            total_items,
            db_size_bytes,
        }),
        Ok(IpcResponse::Error { message }) => Err(message),
        Err(e) => Err(format!("Connection failed: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

/// Clear all non-pinned items.
#[tauri::command]
async fn clear_all() -> Result<String, String> {
    let socket = AppConfig::socket_path();
    let request = IpcRequest::Clear;

    match send_request(&socket, &request).await {
        Ok(IpcResponse::Ok { message }) => Ok(message),
        Ok(IpcResponse::Error { message }) => Err(message),
        Err(e) => Err(format!("Connection failed: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

/// Get the current configuration.
#[tauri::command]
async fn get_config() -> Result<AppConfig, String> {
    let socket = AppConfig::socket_path();
    let request = IpcRequest::GetConfig;

    match send_request(&socket, &request).await {
        Ok(IpcResponse::Config(config)) => Ok(config),
        Ok(IpcResponse::Error { message }) => Err(message),
        // Fallback: load directly from file when daemon doesn't support GetConfig
        Err(_) => Ok(AppConfig::load()),
        _ => Ok(AppConfig::load()),
    }
}

/// Save configuration.
#[tauri::command]
async fn save_config(config: AppConfig) -> Result<String, String> {
    let socket = AppConfig::socket_path();
    let request = IpcRequest::SaveConfig { config };

    match send_request(&socket, &request).await {
        Ok(IpcResponse::Ok { message }) => Ok(message),
        Ok(IpcResponse::Error { message }) => Err(message),
        Err(e) => Err(format!("Connection failed: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

/// Get a base64-encoded image for preview (#38).
#[tauri::command]
async fn get_image_base64(path: String) -> Result<String, String> {
    let file_path = std::path::Path::new(&path);
    if !file_path.exists() {
        return Err("Image file not found".to_string());
    }
    let bytes = std::fs::read(file_path).map_err(|e| format!("Read failed: {}", e))?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:image/png;base64,{}", encoded))
}

/// Add a tag to an item.
#[tauri::command]
async fn add_tag(id: String, tag: String) -> Result<ClipboardItem, String> {
    let socket = AppConfig::socket_path();
    let request = IpcRequest::AddTag { id, tag };

    match send_request(&socket, &request).await {
        Ok(IpcResponse::Item(item)) => Ok(item),
        Ok(IpcResponse::Error { message }) => Err(message),
        Err(e) => Err(format!("Connection failed: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

/// Remove a tag from an item.
#[tauri::command]
async fn remove_tag(id: String, tag: String) -> Result<ClipboardItem, String> {
    let socket = AppConfig::socket_path();
    let request = IpcRequest::RemoveTag { id, tag };

    match send_request(&socket, &request).await {
        Ok(IpcResponse::Item(item)) => Ok(item),
        Ok(IpcResponse::Error { message }) => Err(message),
        Err(e) => Err(format!("Connection failed: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

/// Position the window based on the configured mode.
///
/// - `"mouse"` → spawn near cursor on the active monitor, clamped to screen edges
/// - `"fixed"` → center on the monitor that contains the cursor (or primary fallback)
fn position_window(window: &tauri::WebviewWindow) {
    let config = AppConfig::load();
    let w = config.ui.window_width as i32;
    let h = config.ui.window_height as i32;
    let mode = config.ui.window_position.as_str();

    if let Ok(cursor) = window.cursor_position() {
        if let Ok(monitors) = window.available_monitors() {
            for mon in monitors {
                let pos = mon.position();
                let size = mon.size();
                let right = pos.x + size.width as i32;
                let bottom = pos.y + size.height as i32;

                if (cursor.x as i32) >= pos.x
                    && (cursor.x as i32) < right
                    && (cursor.y as i32) >= pos.y
                    && (cursor.y as i32) < bottom
                {
                    let (x, y) = if mode == "mouse" {
                        // Place near cursor, offset slightly, clamp to screen edges
                        let raw_x = cursor.x as i32 - w / 2;
                        let raw_y = cursor.y as i32 + 20; // 20px below cursor
                        let cx = raw_x.max(pos.x + 8).min(right - w - 8);
                        let cy = raw_y.max(pos.y + 8).min(bottom - h - 8);
                        (cx, cy)
                    } else {
                        // "fixed" — center on the monitor
                        let cx = pos.x + (size.width as i32 - w) / 2;
                        let cy = pos.y + (size.height as i32 - h) / 2;
                        (cx, cy)
                    };

                    let _ = window
                        .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
                    return;
                }
            }
        }
    }

    // Fallback: center on current monitor
    let _ = window.center();
}

/// Refresh the tray menu with the latest 5 clipboard items.
async fn refresh_tray_menu(app: &tauri::AppHandle) {
    use shared::models::IpcResponse;
    use tauri::menu::{MenuBuilder, MenuItemBuilder};

    let socket = AppConfig::socket_path();
    let request = IpcRequest::List {
        offset: 0,
        limit: 5,
    };

    let items = match send_request(&socket, &request).await {
        Ok(IpcResponse::Items { items, .. }) => items,
        _ => return,
    };

    // Build the menu
    let Ok(show_item) = MenuItemBuilder::with_id("toggle", "📋 Show / Hide").build(app) else {
        return;
    };
    let Ok(quit_item) = MenuItemBuilder::with_id("quit", "❌ Quit").build(app) else {
        return;
    };

    let mut builder = MenuBuilder::new(app).item(&show_item).separator();

    for item in &items {
        let type_icon = match item.content_type.as_str() {
            "plain_text" => "📝",
            "html" => "🌐",
            "image" => "🖼️",
            _ => "📋",
        };

        // Truncate preview for menu
        let preview: String = item
            .preview_text
            .replace('\n', " ↵ ")
            .chars()
            .take(45)
            .collect();
        let label = format!("{} {}", type_icon, preview.trim());
        let menu_id = format!("paste_{}", item.id);

        if let Ok(mi) = MenuItemBuilder::with_id(menu_id, label).build(app) {
            builder = builder.item(&mi);
        }
    }

    let Ok(menu) = builder.separator().item(&quit_item).build() else {
        return;
    };

    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_menu(Some(menu));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_items,
            search_items,
            paste_item,
            paste_raw_text,
            pin_item,
            delete_item,
            get_status,
            clear_all,
            get_config,
            save_config,
            get_image_base64,
            add_tag,
            remove_tag,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // --- System Tray ---
            // NOTE: On Linux with AppIndicator, a menu is REQUIRED for the icon to appear.
            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            use tauri::tray::TrayIconBuilder;

            // Build initial menu (will be refreshed with actual items shortly)
            let show_item = MenuItemBuilder::with_id("toggle", "📋 Show / Hide").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "❌ Quit").build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let icon = app.default_window_icon().cloned().unwrap_or_else(|| {
                tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png"))
                    .expect("Failed to load tray icon")
            });

            let tray_window = window.clone();
            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .menu(&tray_menu)
                .tooltip("LinVClipBoard")
                .show_menu_on_left_click(true)
                .on_menu_event(move |app, event| {
                    let id = event.id().as_ref().to_string();
                    match id.as_str() {
                        "toggle" => {
                            if let Some(win) = app.get_webview_window("main") {
                                if let Ok(visible) = win.is_visible() {
                                    if visible {
                                        let _ = win.hide();
                                    } else {
                                        let _ = win.show();
                                        let _ = win.set_focus();
                                    }
                                }
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ if id.starts_with("paste_") => {
                            let item_id = id.strip_prefix("paste_").unwrap().to_string();
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let socket = AppConfig::socket_path();
                                let request = IpcRequest::Paste { id: item_id };
                                let _ = send_request(&socket, &request).await;
                                // Show window briefly for feedback, then hide
                                if let Some(win) = app_handle.get_webview_window("main") {
                                    let _ = win.show();
                                    let _ = win.set_focus();
                                    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                                    let _ = win.hide();
                                }
                            });
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(move |_tray, event| {
                    // Fallback for non-AppIndicator DEs (e.g., KDE, Sway)
                    use tauri::tray::TrayIconEvent;
                    if let TrayIconEvent::Click { .. } = event {
                        if let Ok(visible) = tray_window.is_visible() {
                            if visible {
                                let _ = tray_window.hide();
                            } else {
                                let _ = tray_window.show();
                                let _ = tray_window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // --- Background task: refresh tray menu with latest 5 items ---
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    refresh_tray_menu(&app_handle).await;
                }
            });

            // --- Intercept window close → hide to tray instead of quitting ---
            let close_window = app.get_webview_window("main").unwrap();
            let close_win_ref = close_window.clone();
            close_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = close_win_ref.hide();
                }
            });

            // --- Global Shortcut: configurable (defaults to Super+.) (#21) ---
            use tauri_plugin_global_shortcut::ShortcutState;

            let config = AppConfig::load();
            let shortcut_str = config.ui.shortcut.clone();

            let sc_win = app.get_webview_window("main").unwrap();
            if let Err(e) = app.global_shortcut().on_shortcut(
                shortcut_str.as_str(),
                move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Ok(visible) = sc_win.is_visible() {
                            if visible {
                                let _ = sc_win.hide();
                            } else {
                                position_window(&sc_win);
                                let _ = sc_win.show();
                                let _ = sc_win.set_focus();
                            }
                        }
                    }
                },
            ) {
                eprintln!(
                    "Note: Could not register shortcut '{}': {}",
                    shortcut_str, e
                );
            }

            // --- Background task: push clipboard-updated events to the UI (#13) ---
            let event_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut last_total: u64 = 0;
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    let socket = AppConfig::socket_path();
                    let req = IpcRequest::Status;
                    if let Ok(IpcResponse::Status { total_items, .. }) =
                        send_request(&socket, &req).await
                    {
                        if total_items != last_total {
                            last_total = total_items;
                            let _ = event_handle.emit("clipboard-updated", total_items);
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running LinVClipBoard");
}
