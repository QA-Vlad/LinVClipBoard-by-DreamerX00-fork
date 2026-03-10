use rusqlite::Connection;

/// Current schema version. Bump this when adding migrations.
const CURRENT_SCHEMA_VERSION: u32 = 3;

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

    // ── Migration 3: v2.1.0 snippets table ──────────────────────────────
    if current < 3 {
        tracing::info!("Migration v3: creating snippets table");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS snippets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                folder TEXT DEFAULT '',
                abbreviation TEXT DEFAULT '',
                variables TEXT DEFAULT '[]',
                use_count INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_snippets_folder ON snippets(folder);
            CREATE INDEX IF NOT EXISTS idx_snippets_abbreviation ON snippets(abbreviation);",
        )?;
        // Seed 5 production-ready snippet templates
        conn.execute_batch(
            "INSERT OR IGNORE INTO snippets (id, name, content, folder, abbreviation, variables, use_count, created_at, updated_at) VALUES
            ('seed-email-reply', 'Email Reply', 'Hi {{name}},

Thank you for reaching out. I''ve reviewed your message regarding {{topic}}.

{{response}}

Best regards,
{{sender}}', 'Email', '/reply', '[{\"name\":\"name\",\"default\":\"\"},{\"name\":\"topic\",\"default\":\"\"},{\"name\":\"response\",\"default\":\"\"},{\"name\":\"sender\",\"default\":\"\"}]', 0, datetime('now'), datetime('now')),

            ('seed-bug-report', 'Bug Report', '## Bug Report

**Summary:** {{summary}}

**Steps to Reproduce:**
1. {{step1}}
2. {{step2}}
3. {{step3}}

**Expected Behavior:** {{expected}}
**Actual Behavior:** {{actual}}

**Environment:** {{environment}}', 'Development', '/bug', '[{\"name\":\"summary\",\"default\":\"\"},{\"name\":\"step1\",\"default\":\"\"},{\"name\":\"step2\",\"default\":\"\"},{\"name\":\"step3\",\"default\":\"\"},{\"name\":\"expected\",\"default\":\"\"},{\"name\":\"actual\",\"default\":\"\"},{\"name\":\"environment\",\"default\":\"Linux\"}]', 0, datetime('now'), datetime('now')),

            ('seed-meeting-notes', 'Meeting Notes', '# Meeting Notes — {{date}}

**Attendees:** {{attendees}}
**Topic:** {{topic}}

## Discussion
{{notes}}

## Action Items
- [ ] {{action1}}
- [ ] {{action2}}

## Next Meeting
{{next_date}}', 'Work', '/meeting', '[{\"name\":\"date\",\"default\":\"\"},{\"name\":\"attendees\",\"default\":\"\"},{\"name\":\"topic\",\"default\":\"\"},{\"name\":\"notes\",\"default\":\"\"},{\"name\":\"action1\",\"default\":\"\"},{\"name\":\"action2\",\"default\":\"\"},{\"name\":\"next_date\",\"default\":\"\"}]', 0, datetime('now'), datetime('now')),

            ('seed-code-review', 'Code Review Comment', '### Code Review — {{file}}

**Severity:** {{severity}}

**Issue:**
{{issue}}

**Suggestion:**
```
{{suggestion}}
```

**Why:** {{reason}}', 'Development', '/review', '[{\"name\":\"file\",\"default\":\"\"},{\"name\":\"severity\",\"default\":\"minor\"},{\"name\":\"issue\",\"default\":\"\"},{\"name\":\"suggestion\",\"default\":\"\"},{\"name\":\"reason\",\"default\":\"\"}]', 0, datetime('now'), datetime('now')),

            ('seed-quick-note', 'Quick Note', '📌 {{title}}

{{content}}

— {{date}}', 'Personal', '/note', '[{\"name\":\"title\",\"default\":\"\"},{\"name\":\"content\",\"default\":\"\"},{\"name\":\"date\",\"default\":\"\"}]', 0, datetime('now'), datetime('now'));"
        )?;

        conn.execute("INSERT INTO schema_version (version) VALUES (?1)", [3])?;
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
        assert_eq!(v, 3);

        // Second run — should be a no-op
        run_migrations(&conn).unwrap();
        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 3); // three version rows recorded
    }
}
