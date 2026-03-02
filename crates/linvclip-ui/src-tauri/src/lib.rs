use shared::config::AppConfig;
use shared::ipc::send_request;
use shared::models::{IpcRequest, IpcResponse, ClipboardItem};
use serde::{Deserialize, Serialize};
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

/// Refresh the tray menu with the latest 5 clipboard items.
async fn refresh_tray_menu(app: &tauri::AppHandle) {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    use shared::models::IpcResponse;

    let socket = AppConfig::socket_path();
    let request = IpcRequest::List { offset: 0, limit: 5 };

    let items = match send_request(&socket, &request).await {
        Ok(IpcResponse::Items { items, .. }) => items,
        _ => return,
    };

    // Build the menu
    let Ok(show_item) = MenuItemBuilder::with_id("toggle", "📋 Show / Hide").build(app) else { return };
    let Ok(quit_item) = MenuItemBuilder::with_id("quit", "❌ Quit").build(app) else { return };

    let mut builder = MenuBuilder::new(app)
        .item(&show_item)
        .separator();

    for item in &items {
        let type_icon = match item.content_type.as_str() {
            "plain_text" => "📝",
            "html" => "🌐",
            "image" => "🖼️",
            _ => "📋",
        };

        // Truncate preview for menu
        let preview: String = item.preview_text
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

    let Ok(menu) = builder.separator().item(&quit_item).build() else { return };

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
            pin_item,
            delete_item,
            get_status,
            clear_all,
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

            let icon = app.default_window_icon().cloned()
                .unwrap_or_else(|| tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png")).expect("Failed to load tray icon"));

            let tray_window = window.clone();
            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .menu(&tray_menu)
                .tooltip("LinVClipBoard")
                .menu_on_left_click(true)
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

            // --- Global Shortcut: Super+Period (Win+.) ---
            use tauri_plugin_global_shortcut::ShortcutState;

            let sc_win = app.get_webview_window("main").unwrap();
            if let Err(e) = app.global_shortcut().on_shortcut("Super+.", move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if let Ok(visible) = sc_win.is_visible() {
                        if visible {
                            let _ = sc_win.hide();
                        } else {
                            let _ = sc_win.show();
                            let _ = sc_win.set_focus();
                        }
                    }
                }
            }) {
                eprintln!("Note: Could not register Super+.: {}", e);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running LinVClipBoard");
}
