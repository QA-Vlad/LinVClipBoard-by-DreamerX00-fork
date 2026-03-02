use crate::models::{ClipboardItem, ContentType};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Pool error: {0}")]
    Pool(#[from] r2d2::Error),
    #[error("Item not found: {0}")]
    NotFound(String),
}

pub type DbResult<T> = std::result::Result<T, DbError>;

/// Database manager for clipboard items.
#[derive(Clone)]
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    /// Open or create the database.
    pub fn open(path: &Path) -> DbResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::builder().max_size(8).build(manager)?;

        // Initialize schema on one connection
        {
            let conn = pool.get()?;

            // Performance pragmas
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA cache_size = 8000;
                 PRAGMA temp_store = MEMORY;",
            )?;

            // Create tables
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS clipboard_items (
                    id TEXT PRIMARY KEY,
                    content_type TEXT NOT NULL,
                    content TEXT NOT NULL,
                    preview_text TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL,
                    pinned INTEGER NOT NULL DEFAULT 0,
                    tags TEXT NOT NULL DEFAULT '[]',
                    app_source TEXT,
                    checksum TEXT NOT NULL,
                    size_bytes INTEGER NOT NULL DEFAULT 0
                );

                CREATE INDEX IF NOT EXISTS idx_created_at ON clipboard_items(created_at DESC);
                CREATE INDEX IF NOT EXISTS idx_pinned ON clipboard_items(pinned);
                CREATE UNIQUE INDEX IF NOT EXISTS idx_checksum ON clipboard_items(checksum);

                CREATE VIRTUAL TABLE IF NOT EXISTS clipboard_fts USING fts5(
                    preview_text,
                    content='clipboard_items',
                    content_rowid='rowid'
                );

                CREATE TRIGGER IF NOT EXISTS clipboard_fts_insert AFTER INSERT ON clipboard_items BEGIN
                    INSERT INTO clipboard_fts(rowid, preview_text) VALUES (new.rowid, new.preview_text);
                END;

                CREATE TRIGGER IF NOT EXISTS clipboard_fts_delete AFTER DELETE ON clipboard_items BEGIN
                    INSERT INTO clipboard_fts(clipboard_fts, rowid, preview_text) VALUES ('delete', old.rowid, old.preview_text);
                END;

                CREATE TRIGGER IF NOT EXISTS clipboard_fts_update AFTER UPDATE ON clipboard_items BEGIN
                    INSERT INTO clipboard_fts(clipboard_fts, rowid, preview_text) VALUES ('delete', old.rowid, old.preview_text);
                    INSERT INTO clipboard_fts(rowid, preview_text) VALUES (new.rowid, new.preview_text);
                END;",
            )?;

            // Run schema migrations
            crate::migration::run_migrations(&conn).map_err(DbError::Sqlite)?;
        }

        Ok(Self { pool })
    }

    /// Insert a new clipboard item. Returns false if duplicate checksum found.
    /// If a duplicate is found, bump it to the top by updating created_at.
    pub fn insert(&self, item: &ClipboardItem) -> DbResult<bool> {
        let conn = self.pool.get()?;

        // Atomic: try to bump existing duplicate first
        let bumped = conn.execute(
            "UPDATE clipboard_items SET created_at = ?1 WHERE checksum = ?2",
            params![item.created_at.to_rfc3339(), item.checksum],
        )?;

        if bumped > 0 {
            tracing::debug!(
                "Bumped duplicate item to top (checksum={})",
                &item.checksum[..8]
            );
            return Ok(false);
        }

        // No duplicate — insert. UNIQUE index on checksum guards against races.
        let result = conn.execute(
            "INSERT OR IGNORE INTO clipboard_items (id, content_type, content, preview_text, created_at, pinned, app_source, checksum, size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                item.id,
                item.content_type.as_str(),
                item.content,
                item.preview_text,
                item.created_at.to_rfc3339(),
                item.pinned as i32,
                item.app_source,
                item.checksum,
                item.size_bytes as i64,
            ],
        )?;

        if result == 0 {
            // Race: another connection inserted the same checksum between our UPDATE and INSERT
            tracing::debug!(
                "Duplicate caught by UNIQUE constraint (checksum={})",
                &item.checksum[..8]
            );
            return Ok(false);
        }

        tracing::debug!(
            "Inserted item: {} ({})",
            item.id,
            item.content_type.as_str()
        );
        Ok(true)
    }

    /// List items with pagination, pinned items first.
    pub fn list(&self, offset: u32, limit: u32) -> DbResult<(Vec<ClipboardItem>, u64)> {
        let conn = self.pool.get()?;

        let total: u64 =
            conn.query_row("SELECT COUNT(*) FROM clipboard_items", [], |row| row.get(0))?;

        let mut stmt = conn.prepare(
            "SELECT id, content_type, content, preview_text, created_at, pinned, app_source, checksum, size_bytes
             FROM clipboard_items
             ORDER BY pinned DESC, created_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let items = stmt
            .query_map(params![limit, offset], |row| Ok(row_to_item(row)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok((items, total))
    }

    /// Search items using FTS5 full-text search.
    pub fn search(&self, query: &str, limit: u32) -> DbResult<(Vec<ClipboardItem>, u64)> {
        let conn = self.pool.get()?;

        // Escape ALL FTS5 special characters, then wrap as a phrase prefix query.
        let sanitized: String = query
            .chars()
            .filter(|c| !matches!(c, '"' | '*' | '^' | ':' | '+' | '-' | '(' | ')' | '{' | '}'))
            .collect();
        let fts_query = format!("\"{}\"*", sanitized);

        let total: u64 = conn.query_row(
            "SELECT COUNT(*) FROM clipboard_items c
             JOIN clipboard_fts f ON c.rowid = f.rowid
             WHERE clipboard_fts MATCH ?1",
            params![&fts_query],
            |row| row.get(0),
        )?;

        let mut stmt = conn.prepare(
            "SELECT c.id, c.content_type, c.content, c.preview_text, c.created_at, c.pinned, c.app_source, c.checksum, c.size_bytes
             FROM clipboard_items c
             JOIN clipboard_fts f ON c.rowid = f.rowid
             WHERE clipboard_fts MATCH ?1
             ORDER BY c.pinned DESC, c.created_at DESC
             LIMIT ?2",
        )?;

        let items = stmt
            .query_map(params![fts_query, limit], |row| Ok(row_to_item(row)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok((items, total))
    }

    /// Get a single item by ID.
    pub fn get(&self, id: &str) -> DbResult<ClipboardItem> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare(
            "SELECT id, content_type, content, preview_text, created_at, pinned, app_source, checksum, size_bytes
             FROM clipboard_items WHERE id = ?1",
        )?;

        stmt.query_row(params![id], |row| Ok(row_to_item(row)))
            .map_err(|_| DbError::NotFound(id.to_string()))
    }

    /// Delete an item by ID. Removes associated blob file if it exists.
    pub fn delete(&self, id: &str) -> DbResult<()> {
        // Get item first to check for blob file
        if let Ok(item) = self.get(id) {
            if item.content_type == ContentType::Image {
                let blob_path = std::path::Path::new(&item.content);
                if blob_path.exists() {
                    let _ = std::fs::remove_file(blob_path);
                }
            }
        }

        let conn = self.pool.get()?;
        conn.execute("DELETE FROM clipboard_items WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Bulk delete items inside a single transaction.
    pub fn bulk_delete(&self, ids: &[String]) -> DbResult<u64> {
        // Collect blob paths before deletion so we can remove files after commit.
        let blob_paths: Vec<String> = ids
            .iter()
            .filter_map(|id| self.get(id).ok())
            .filter(|item| item.content_type == ContentType::Image)
            .map(|item| item.content.clone())
            .collect();

        let conn = self.pool.get()?;
        let tx = conn.unchecked_transaction()?;
        let mut count = 0u64;
        for id in ids {
            count += tx.execute("DELETE FROM clipboard_items WHERE id = ?1", params![id])? as u64;
        }
        tx.commit()?;

        // Clean up blob files outside the transaction.
        for path_str in &blob_paths {
            let path = std::path::Path::new(path_str);
            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
        }

        Ok(count)
    }

    /// Toggle the pinned state of an item.
    pub fn toggle_pin(&self, id: &str) -> DbResult<ClipboardItem> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE clipboard_items SET pinned = NOT pinned WHERE id = ?1",
            params![id],
        )?;
        drop(conn);
        self.get(id)
    }

    /// Clear all non-pinned items.
    pub fn clear_unpinned(&self) -> DbResult<u64> {
        let conn = self.pool.get()?;

        // Delete blob files for images
        let mut stmt = conn.prepare(
            "SELECT content FROM clipboard_items WHERE pinned = 0 AND content_type = 'image'",
        )?;
        let blobs: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        for blob_path in &blobs {
            let path = std::path::Path::new(blob_path);
            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
        }

        let deleted = conn.execute("DELETE FROM clipboard_items WHERE pinned = 0", [])?;
        Ok(deleted as u64)
    }

    /// Remove blob files on disk that are not referenced by any database row.
    pub fn cleanup_orphan_blobs(&self, blob_dir: &Path) -> DbResult<u64> {
        let conn = self.pool.get()?;

        let mut stmt =
            conn.prepare("SELECT content FROM clipboard_items WHERE content_type = 'image'")?;
        let known: std::collections::HashSet<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let mut removed = 0u64;
        if let Ok(entries) = std::fs::read_dir(blob_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let path_str = path.to_string_lossy().to_string();
                    if !known.contains(&path_str) && std::fs::remove_file(&path).is_ok() {
                        removed += 1;
                    }
                }
            }
        }

        if removed > 0 {
            tracing::info!("Cleaned up {} orphan blob files", removed);
        }
        Ok(removed)
    }

    /// Enforce storage limits: remove oldest non-pinned items beyond max_items.
    pub fn enforce_limits(&self, config: &crate::config::StorageConfig) -> DbResult<()> {
        let conn = self.pool.get()?;

        let total: u64 =
            conn.query_row("SELECT COUNT(*) FROM clipboard_items", [], |row| row.get(0))?;

        if total > config.max_items {
            let to_remove = total - config.max_items;
            // Collect blob paths for image items that will be removed.
            let mut blob_stmt = conn.prepare(
                "SELECT content FROM clipboard_items
                 WHERE pinned = 0 AND content_type = 'image'
                 ORDER BY created_at ASC LIMIT ?1",
            )?;
            let blobs: Vec<String> = blob_stmt
                .query_map(params![to_remove as i64], |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?;

            conn.execute(
                "DELETE FROM clipboard_items WHERE id IN (
                    SELECT id FROM clipboard_items WHERE pinned = 0
                    ORDER BY created_at ASC LIMIT ?1
                )",
                params![to_remove as i64],
            )?;

            // Remove blob files for evicted images.
            for blob_path in &blobs {
                let p = std::path::Path::new(blob_path);
                if p.exists() {
                    let _ = std::fs::remove_file(p);
                }
            }

            tracing::info!("Enforced limits: removed {} items", to_remove);
        }

        // Remove expired items (use TimeDelta to avoid deprecation).
        let expiry_delta = chrono::TimeDelta::try_days(config.expiry_days as i64)
            .unwrap_or_else(|| chrono::TimeDelta::days(30));
        let expiry_date = chrono::Utc::now() - expiry_delta;

        // Collect blob paths for expired images.
        let mut exp_blob_stmt = conn.prepare(
            "SELECT content FROM clipboard_items
             WHERE pinned = 0 AND content_type = 'image' AND created_at < ?1",
        )?;
        let expired_blobs: Vec<String> = exp_blob_stmt
            .query_map(params![expiry_date.to_rfc3339()], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        let removed = conn.execute(
            "DELETE FROM clipboard_items WHERE pinned = 0 AND created_at < ?1",
            params![expiry_date.to_rfc3339()],
        )?;

        for blob_path in &expired_blobs {
            let p = std::path::Path::new(blob_path);
            if p.exists() {
                let _ = std::fs::remove_file(p);
            }
        }

        if removed > 0 {
            tracing::info!("Removed {} expired items", removed);
        }

        Ok(())
    }

    /// Get the total count of items.
    pub fn total_items(&self) -> DbResult<u64> {
        let conn = self.pool.get()?;
        let count: u64 =
            conn.query_row("SELECT COUNT(*) FROM clipboard_items", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Get the database file size in bytes.
    pub fn db_size(&self) -> DbResult<u64> {
        let conn = self.pool.get()?;
        let size: i64 = conn.query_row(
            "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
            [],
            |row| row.get(0),
        )?;
        Ok(size as u64)
    }

    /// Check if a checksum already exists (for dedup before insert).
    pub fn has_checksum(&self, checksum: &str) -> DbResult<bool> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM clipboard_items WHERE checksum = ?1",
            params![checksum],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Add a tag to an item.
    pub fn add_tag(&self, id: &str, tag: &str) -> DbResult<ClipboardItem> {
        let conn = self.pool.get()?;
        let tags_json: String = conn
            .query_row(
                "SELECT tags FROM clipboard_items WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|_| DbError::NotFound(id.to_string()))?;

        let mut tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        let tag_str = tag.to_string();
        if !tags.contains(&tag_str) {
            tags.push(tag_str);
        }
        let new_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "UPDATE clipboard_items SET tags = ?1 WHERE id = ?2",
            params![new_json, id],
        )?;
        drop(conn);
        self.get(id)
    }

    /// Remove a tag from an item.
    pub fn remove_tag(&self, id: &str, tag: &str) -> DbResult<ClipboardItem> {
        let conn = self.pool.get()?;
        let tags_json: String = conn
            .query_row(
                "SELECT tags FROM clipboard_items WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|_| DbError::NotFound(id.to_string()))?;

        let mut tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        tags.retain(|t| t != tag);
        let new_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "UPDATE clipboard_items SET tags = ?1 WHERE id = ?2",
            params![new_json, id],
        )?;
        drop(conn);
        self.get(id)
    }
}

/// Convert a rusqlite row to a ClipboardItem.
fn row_to_item(row: &rusqlite::Row) -> ClipboardItem {
    let created_str: String = row.get(4).unwrap_or_default();
    let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());

    ClipboardItem {
        id: row.get(0).unwrap_or_default(),
        content_type: row
            .get::<_, String>(1)
            .unwrap_or_default()
            .parse::<ContentType>()
            .unwrap_or(ContentType::PlainText),
        content: row.get(2).unwrap_or_default(),
        preview_text: row.get(3).unwrap_or_default(),
        created_at,
        pinned: row.get::<_, i32>(5).unwrap_or(0) != 0,
        app_source: row.get(6).ok(),
        checksum: row.get(7).unwrap_or_default(),
        size_bytes: row.get::<_, i64>(8).unwrap_or(0) as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ContentType;
    use std::path::PathBuf;

    fn create_test_db() -> (Database, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        (db, dir)
    }

    fn make_item(content: &str) -> ClipboardItem {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let checksum = hex::encode(hasher.finalize());

        ClipboardItem::new(
            ContentType::PlainText,
            content.to_string(),
            content.chars().take(200).collect(),
            checksum,
            content.len() as u64,
        )
    }

    #[test]
    fn test_insert_and_list() {
        let (db, _dir) = create_test_db();
        let item = make_item("Hello, world!");
        assert!(db.insert(&item).unwrap());

        let (items, total) = db.list(0, 10).unwrap();
        assert_eq!(total, 1);
        assert_eq!(items[0].content, "Hello, world!");
    }

    #[test]
    fn test_deduplication() {
        let (db, _dir) = create_test_db();
        let item1 = make_item("duplicate text");
        assert!(db.insert(&item1).unwrap());
        let item2 = make_item("duplicate text");
        assert!(!db.insert(&item2).unwrap()); // Should be skipped

        let (_, total) = db.list(0, 10).unwrap();
        assert_eq!(total, 1);
    }

    #[test]
    fn test_search() {
        let (db, _dir) = create_test_db();
        db.insert(&make_item("rust programming language")).unwrap();
        db.insert(&make_item("python scripting")).unwrap();
        db.insert(&make_item("rusty old car")).unwrap();

        let (items, _) = db.search("rust", 10).unwrap();
        assert!(items.len() >= 1);
    }

    #[test]
    fn test_pin_toggle() {
        let (db, _dir) = create_test_db();
        let item = make_item("pin me");
        db.insert(&item).unwrap();

        let pinned = db.toggle_pin(&item.id).unwrap();
        assert!(pinned.pinned);

        let unpinned = db.toggle_pin(&item.id).unwrap();
        assert!(!unpinned.pinned);
    }

    #[test]
    fn test_delete() {
        let (db, _dir) = create_test_db();
        let item = make_item("delete me");
        db.insert(&item).unwrap();

        db.delete(&item.id).unwrap();
        let (_, total) = db.list(0, 10).unwrap();
        assert_eq!(total, 0);
    }

    #[test]
    fn test_clear_unpinned() {
        let (db, _dir) = create_test_db();
        let item1 = make_item("keep me");
        db.insert(&item1).unwrap();
        db.toggle_pin(&item1.id).unwrap();

        let item2 = make_item("remove me");
        db.insert(&item2).unwrap();

        db.clear_unpinned().unwrap();
        let (items, total) = db.list(0, 10).unwrap();
        assert_eq!(total, 1);
        assert_eq!(items[0].id, item1.id);
    }
}
