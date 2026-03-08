use rusqlite::Connection;

/// Current schema version. Bump this when adding migrations.
const CURRENT_SCHEMA_VERSION: u32 = 2;

/// Run all pending migrations on the given connection.
///
/// This must be called AFTER the base tables are created in `Database::open`,
/// so that migration 1 (baseline) can be recorded immediately.
pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    // Create the version-tracking table if it doesn't exist.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version  INTEGER NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let current: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current >= CURRENT_SCHEMA_VERSION {
        tracing::debug!("Schema is up-to-date (v{})", current);
        return Ok(());
    }

    tracing::info!(
        "Running database migrations: v{} → v{}",
        current,
        CURRENT_SCHEMA_VERSION
    );

    // ── Migration 1: baseline ────────────────────────────────────────────
    // The core tables are created in Database::open(). This migration just
    // records that the baseline schema is in place.
    if current < 1 {
        tracing::info!("Migration v1: recording baseline schema");
        conn.execute("INSERT INTO schema_version (version) VALUES (?1)", [1])?;
    }

    // ── Migration 2: v2.0.0 rich content types ────────────────────────────
    // No schema change needed — content_type TEXT already supports Html/Files/Uri.
    // This migration just records the version bump.
    if current < 2 {
        tracing::info!("Migration v2: recording v2.0.0 rich content types");
        conn.execute("INSERT INTO schema_version (version) VALUES (?1)", [2])?;
    }

    tracing::info!("Migrations complete — now at v{}", CURRENT_SCHEMA_VERSION);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migration_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        // Create baseline tables so migration can reference them
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
            );",
        )
        .unwrap();

        // First run
        run_migrations(&conn).unwrap();
        let v: u32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, 2);

        // Second run — should be a no-op
        run_migrations(&conn).unwrap();
        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2); // two version rows recorded
    }
}
