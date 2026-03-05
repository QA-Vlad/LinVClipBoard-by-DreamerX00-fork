use base64::Engine;
use serde::{Deserialize, Serialize};
use shared::config::AppConfig;
use shared::ipc::send_request;
use shared::models::{ClipboardItem, IpcRequest, IpcResponse};
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

// Obfuscated KLIPY API key generated at build time (XOR-scrambled).
include!(concat!(env!("OUT_DIR"), "/klipy_key.rs"));

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

#[derive(Serialize, Deserialize, Clone)]
pub struct GifItem {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub preview_url: String,
    pub gif_url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize)]
pub struct GifResult {
    pub items: Vec<GifItem>,
    pub page: u32,
    pub has_next: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GifCategory {
    pub category: String,
    pub query: String,
    pub preview_url: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateInfo {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
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

/// Type text into the previously focused application (like the Windows emoji panel).
///
/// 1. Hides the window so focus returns to the previous app.
/// 2. Tries to type the text using `wtype` (Wayland) / `xdotool` / `ydotool`.
/// 3. Falls back to clipboard copy if no typing tool is available.
///
/// Returns `"typed"` on success, `"copied"` when falling back to clipboard.
#[tauri::command]
async fn type_text(text: String, app: tauri::AppHandle) -> Result<String, String> {
    // Hide window → focus returns to the previous application
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.hide();
    }

    // Allow the compositor to transfer focus
    tokio::time::sleep(std::time::Duration::from_millis(120)).await;

    // Try direct typing first (doesn't touch the clipboard)
    if try_type_direct(&text) {
        return Ok("typed".to_string());
    }

    // Fallback: copy to clipboard, then try simulating Ctrl+V
    let mut clipboard = arboard::Clipboard::new().map_err(|e| format!("Clipboard error: {}", e))?;
    clipboard
        .set_text(&text)
        .map_err(|e| format!("Failed to set clipboard: {}", e))?;

    if try_paste_shortcut() {
        return Ok("typed".to_string());
    }

    // Last resort: text is on the clipboard, user can Ctrl+V manually
    Ok("copied".to_string())
}

/// Try to type text directly into the currently focused window.
fn try_type_direct(text: &str) -> bool {
    // wtype — works on wlroots-based Wayland compositors (Sway, Hyprland, etc.)
    if run_silent("wtype", &["--", text]) {
        return true;
    }
    // xdotool — works on X11 and XWayland
    if run_silent("xdotool", &["type", "--clearmodifiers", "--", text]) {
        return true;
    }
    // ydotool — works on both X11 and Wayland (needs ydotoold)
    if run_silent("ydotool", &["type", "--", text]) {
        return true;
    }
    false
}

/// Try to simulate Ctrl+V in the currently focused window.
fn try_paste_shortcut() -> bool {
    if run_silent("wtype", &["-M", "ctrl", "-P", "v", "-p", "v", "-m", "ctrl"]) {
        return true;
    }
    if run_silent("xdotool", &["key", "--clearmodifiers", "ctrl+v"]) {
        return true;
    }
    false
}

/// Run a command silently, returning true if it exits successfully.
fn run_silent(program: &str, args: &[&str]) -> bool {
    std::process::Command::new(program)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
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

    let result = match send_request(&socket, &request).await {
        Ok(IpcResponse::Ok { message }) => Ok(message),
        Ok(IpcResponse::Error { message }) => Err(message),
        Err(e) => Err(format!("Connection failed: {}", e)),
        _ => Err("Unexpected response".to_string()),
    };

    // Also purge the GIF cache
    if let Ok(dir) = gif_cache_dir() {
        if dir.exists() {
            let _ = std::fs::remove_dir_all(&dir);
            let _ = std::fs::create_dir_all(&dir);
        }
    }

    result
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

/// Check GitHub releases for a newer version.
#[tauri::command]
async fn check_for_updates() -> Result<UpdateInfo, String> {
    let current = env!("CARGO_PKG_VERSION");

    let client = reqwest::Client::builder()
        .user_agent("LinVClipBoard")
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let resp = client
        .get("https://api.github.com/repos/DreamerX00/LinVClipBoard/releases/latest")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API error: {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    let tag = body["tag_name"]
        .as_str()
        .ok_or("No tag_name in response")?;
    let latest = tag.trim_start_matches('v');
    let html_url = body["html_url"].as_str().unwrap_or(
        "https://github.com/DreamerX00/LinVClipBoard/releases",
    );

    // Simple semver comparison: split by '.' and compare numerically
    let parse_ver = |s: &str| -> Vec<u32> {
        s.split('.').filter_map(|p| p.parse().ok()).collect()
    };
    let cur_parts = parse_ver(current);
    let lat_parts = parse_ver(latest);
    let has_update = lat_parts > cur_parts;

    Ok(UpdateInfo {
        has_update,
        current_version: current.to_string(),
        latest_version: latest.to_string(),
        release_url: html_url.to_string(),
    })
}

/// Decode the embedded KLIPY app key (XOR-descrambled at runtime).
fn get_gif_api_key() -> Result<String, String> {
    if KLIPY_KEY_BYTES.is_empty() {
        return Err("gif_api_key_missing".to_string());
    }
    let decoded: String = KLIPY_KEY_BYTES
        .iter()
        .enumerate()
        .map(|(i, &b)| (b ^ KLIPY_KEY_XOR_PAD[i % KLIPY_KEY_XOR_PAD.len()]) as char)
        .collect();
    Ok(decoded)
}

/// Helper: parse a KLIPY v1 GIF object from JSON.
fn parse_gif_item(r: &serde_json::Value) -> Option<GifItem> {
    let id = r["id"].as_i64().or_else(|| r["id"].as_u64().map(|v| v as i64))?;
    let slug = r["slug"].as_str().unwrap_or("").to_string();
    let title = r["title"].as_str().unwrap_or("").to_string();

    // Prefer sm.webp (fast, small) → sm.gif → xs.gif for preview
    // Use hd.gif for the URL users copy
    let file = &r["file"];
    let preview_url = file["sm"]["webp"]["url"]
        .as_str()
        .or_else(|| file["sm"]["gif"]["url"].as_str())
        .or_else(|| file["xs"]["gif"]["url"].as_str())?
        .to_string();
    let gif_url = file["hd"]["gif"]["url"]
        .as_str()
        .or_else(|| file["md"]["gif"]["url"].as_str())
        .unwrap_or(preview_url.as_str())
        .to_string();
    let width = file["sm"]["webp"]["width"]
        .as_u64()
        .or_else(|| file["sm"]["gif"]["width"].as_u64())
        .unwrap_or(220) as u32;
    let height = file["sm"]["webp"]["height"]
        .as_u64()
        .or_else(|| file["sm"]["gif"]["height"].as_u64())
        .unwrap_or(220) as u32;

    Some(GifItem {
        id: id.to_string(),
        slug,
        title,
        preview_url,
        gif_url,
        width,
        height,
    })
}

/// Fetch GIFs from the KLIPY v1 API.
///
/// If `query` is empty, fetches trending GIFs.
#[tauri::command]
async fn fetch_gifs(query: String, page: u32, per_page: u32) -> Result<GifResult, String> {
    let app_key = get_gif_api_key()?;

    let client = reqwest::Client::new();
    let (url, is_search) = if query.trim().is_empty() {
        (
            format!("https://api.klipy.com/api/v1/{}/gifs/trending", app_key),
            false,
        )
    } else {
        (
            format!("https://api.klipy.com/api/v1/{}/gifs/search", app_key),
            true,
        )
    };

    let mut params: Vec<(&str, String)> = vec![
        ("page", page.to_string()),
        ("per_page", per_page.to_string()),
        ("customer_id", "linvclipboard_user".to_string()),
        ("content_filter", "medium".to_string()),
        ("format_filter", "gif,webp,jpg".to_string()),
    ];
    if is_search {
        params.push(("q", query));
    }

    let resp = client
        .get(&url)
        .query(&params)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if body["result"].as_bool() != Some(true) {
        return Err("API returned error".to_string());
    }

    let data = &body["data"];
    let has_next = data["has_next"].as_bool().unwrap_or(false);
    let current_page = data["current_page"].as_u64().unwrap_or(page as u64) as u32;
    let results = data["data"].as_array().ok_or("No data array in response")?;

    let items: Vec<GifItem> = results.iter().filter_map(parse_gif_item).collect();

    Ok(GifResult {
        items,
        page: current_page,
        has_next,
    })
}

/// Fetch GIF categories from the KLIPY v1 API.
#[tauri::command]
async fn fetch_gif_categories() -> Result<Vec<GifCategory>, String> {
    let app_key = get_gif_api_key()?;

    let client = reqwest::Client::new();
    let url = format!("https://api.klipy.com/api/v1/{}/gifs/categories", app_key);

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if body["result"].as_bool() != Some(true) {
        return Err("API returned error".to_string());
    }

    let cats = body["data"]["categories"]
        .as_array()
        .ok_or("No categories in response")?;

    let categories: Vec<GifCategory> = cats
        .iter()
        .filter_map(|c| {
            let category = c["category"].as_str()?.to_string();
            let query = c["query"].as_str()?.to_string();
            let preview_url = c["preview_url"].as_str()?.to_string();
            Some(GifCategory {
                category,
                query,
                preview_url,
            })
        })
        .collect();

    Ok(categories)
}

/// Register a GIF share event with KLIPY v1 API (POST).
#[tauri::command]
async fn register_gif_share(slug: String, query: String) -> Result<String, String> {
    let app_key = get_gif_api_key()?;

    let client = reqwest::Client::new();
    let url = format!("https://api.klipy.com/api/v1/{}/gifs/share/{}", app_key, slug);

    let mut body_map = serde_json::Map::new();
    body_map.insert(
        "customer_id".to_string(),
        serde_json::Value::String("linvclipboard_user".to_string()),
    );
    if !query.is_empty() {
        body_map.insert("q".to_string(), serde_json::Value::String(query));
    }

    let _ = client
        .post(&url)
        .json(&serde_json::Value::Object(body_map))
        .send()
        .await;

    Ok("ok".to_string())
}

/// Return the GIF cache directory, creating it if needed.
fn gif_cache_dir() -> Result<std::path::PathBuf, String> {
    let dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("linvclip")
        .join("gifs");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create cache dir: {}", e))?;
    Ok(dir)
}

/// Copy a GIF URL to the system clipboard.
///
/// Worldwide, GIF keyboards on desktop work by copying the direct GIF URL
/// as plain text.  Chat apps (Discord, Telegram, Slack, etc.) auto-embed
/// direct `.gif` links, displaying them as animated images.
///
/// The `image/gif` MIME type is NOT supported by most paste targets (Electron
/// apps read `image/png` or `text/plain`), and `wl-clipboard` cannot offer
/// multiple MIME types simultaneously, so URL-based sharing is the standard.
#[tauri::command]
async fn copy_gif(url: String) -> Result<String, String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Clipboard error: {}", e))?;
    clipboard
        .set_text(&url)
        .map_err(|e| format!("Failed to set clipboard: {}", e))?;
    Ok("ok".to_string())
}

/// Purge all cached GIF files.
#[tauri::command]
async fn clear_gif_cache() -> Result<String, String> {
    if let Ok(dir) = gif_cache_dir() {
        if dir.exists() {
            std::fs::remove_dir_all(&dir)
                .map_err(|e| format!("Failed to clear GIF cache: {}", e))?;
            // Re-create empty dir
            std::fs::create_dir_all(&dir).ok();
        }
    }
    Ok("ok".to_string())
}

/// Purge GIF cache files older than the configured expiry days.
/// Called on app startup.
fn cleanup_expired_gif_cache() {
    let expiry_days = {
        let cfg = AppConfig::load();
        cfg.storage.expiry_days
    };
    let max_age = std::time::Duration::from_secs(expiry_days as u64 * 86400);

    let dir = match gif_cache_dir() {
        Ok(d) => d,
        Err(_) => return,
    };

    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let now = std::time::SystemTime::now();
    for entry in entries.flatten() {
        if let Ok(meta) = entry.metadata() {
            let age = meta.modified().ok().and_then(|m| now.duration_since(m).ok());
            if let Some(age) = age {
                if age > max_age {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }
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
            type_text,
            pin_item,
            delete_item,
            get_status,
            clear_all,
            get_config,
            save_config,
            get_image_base64,
            add_tag,
            remove_tag,
            fetch_gifs,
            fetch_gif_categories,
            register_gif_share,
            copy_gif,
            clear_gif_cache,
            check_for_updates,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // Clean up expired GIF cache files on startup
            std::thread::spawn(|| cleanup_expired_gif_cache());

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
