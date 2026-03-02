use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Content type of a clipboard item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    PlainText,
    Html,
    Image,
    RichText,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::PlainText => "plain_text",
            ContentType::Html => "html",
            ContentType::Image => "image",
            ContentType::RichText => "rich_text",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "plain_text" => ContentType::PlainText,
            "html" => ContentType::Html,
            "image" => ContentType::Image,
            "rich_text" => ContentType::RichText,
            _ => ContentType::PlainText,
        }
    }
}

/// A clipboard history item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub content_type: ContentType,
    /// For text: the full content. For images: path to blob file.
    pub content: String,
    /// Short preview text for display (first 200 chars or image dimensions).
    pub preview_text: String,
    pub created_at: DateTime<Utc>,
    pub pinned: bool,
    pub app_source: Option<String>,
    pub checksum: String,
    /// Content size in bytes.
    pub size_bytes: u64,
}

impl ClipboardItem {
    pub fn new(
        content_type: ContentType,
        content: String,
        preview_text: String,
        checksum: String,
        size_bytes: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content_type,
            content,
            preview_text,
            created_at: Utc::now(),
            pinned: false,
            app_source: None,
            checksum,
            size_bytes,
        }
    }
}

/// IPC request from client to daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum IpcRequest {
    /// List items with pagination.
    List { offset: u32, limit: u32 },
    /// Search items by query.
    Search { query: String, limit: u32 },
    /// Get a single item by ID.
    Get { id: String },
    /// Delete an item by ID.
    Delete { id: String },
    /// Bulk delete items.
    BulkDelete { ids: Vec<String> },
    /// Toggle pin on an item.
    TogglePin { id: String },
    /// Paste an item (set it as current clipboard content).
    Paste { id: String },
    /// Clear all non-pinned items.
    Clear,
    /// Get daemon status.
    Status,
}

/// IPC response from daemon to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum IpcResponse {
    /// List of clipboard items.
    Items {
        items: Vec<ClipboardItem>,
        total: u64,
    },
    /// Single clipboard item.
    Item(ClipboardItem),
    /// Operation success.
    Ok { message: String },
    /// Error response.
    Error { message: String },
    /// Daemon status.
    Status {
        uptime_secs: u64,
        total_items: u64,
        db_size_bytes: u64,
    },
}

/// IPC message envelope with length-prefix framing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage<T> {
    pub payload: T,
}
