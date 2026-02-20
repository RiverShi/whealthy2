use rusqlite::{Connection, Result};

/// 初始化数据库表（仅在表不存在时创建）
pub fn initialize_database(conn: &Connection) -> Result<()> {
    println!("=== Initializing database tables ===");
    let result = conn.execute_batch(r#"
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS books (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            created_at  TEXT NOT NULL,
            archived_at TEXT
        );

        CREATE TABLE IF NOT EXISTS categories (
            id        TEXT PRIMARY KEY,
            domain    TEXT NOT NULL CHECK(domain IN ('asset','liability','income','expense')),
            level     INTEGER NOT NULL CHECK(level IN (1,2)),
            parent_id TEXT REFERENCES categories(id) ON DELETE CASCADE,
            name      TEXT NOT NULL,
            icon      TEXT
        );

        CREATE TABLE IF NOT EXISTS tags (
            id     TEXT PRIMARY KEY,
            domain TEXT NOT NULL CHECK(domain IN ('asset','liability','transaction')),
            name   TEXT NOT NULL,
            color  TEXT
        );

        CREATE TABLE IF NOT EXISTS entries (
            id              TEXT PRIMARY KEY,
            book_id         TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            name            TEXT NOT NULL,
            kind            TEXT NOT NULL CHECK(kind IN ('asset','liability')),
            is_account      INTEGER NOT NULL DEFAULT 0,
            valuation_type  TEXT NOT NULL CHECK(valuation_type IN ('fixed','manual')),
            value           REAL NOT NULL DEFAULT 0,
            category_l1_id  TEXT REFERENCES categories(id),
            category_l2_id  TEXT REFERENCES categories(id),
            extra           TEXT,
            opened_at       TEXT NOT NULL,
            closed_at       TEXT
        );

        CREATE TABLE IF NOT EXISTS entry_tags (
            entry_id TEXT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
            tag_id   TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY (entry_id, tag_id)
        );

        CREATE TABLE IF NOT EXISTS entry_adjustments (
            id          TEXT PRIMARY KEY,
            entry_id    TEXT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
            old_value   REAL NOT NULL,
            new_value   REAL NOT NULL,
            reason      TEXT,
            adjusted_at TEXT NOT NULL
        );

        -- 事件：可选的轻量聚合单元（如"购物+部分退款"、"朋友聚餐AA"）
        -- 不承载时间，时间由各 Record 自己携带
        CREATE TABLE IF NOT EXISTS events (
            id          TEXT PRIMARY KEY,
            book_id     TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            name        TEXT NOT NULL,
            description TEXT,
            created_at  TEXT NOT NULL
        );

        -- 流水记录：原子记账单元，可独立展示，可选归属某个事件
        CREATE TABLE IF NOT EXISTS records (
            id              TEXT PRIMARY KEY,
            book_id         TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            event_id        TEXT REFERENCES events(id) ON DELETE SET NULL,
            type            TEXT NOT NULL CHECK(type IN ('income','expense','transfer')),
            amount          REAL NOT NULL CHECK(amount > 0),
            category_id     TEXT REFERENCES categories(id),
            from_account_id TEXT REFERENCES entries(id),
            to_account_id   TEXT REFERENCES entries(id),
            note            TEXT,
            happened_at     TEXT NOT NULL,
            created_at      TEXT NOT NULL
        );

        -- 流水记录标签关联
        CREATE TABLE IF NOT EXISTS record_tags (
            record_id TEXT NOT NULL REFERENCES records(id) ON DELETE CASCADE,
            tag_id    TEXT NOT NULL REFERENCES tags(id)    ON DELETE CASCADE,
            PRIMARY KEY (record_id, tag_id)
        );

        CREATE TABLE IF NOT EXISTS views (
            id   TEXT PRIMARY KEY,
            name TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS view_books (
            view_id TEXT NOT NULL REFERENCES views(id) ON DELETE CASCADE,
            book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            PRIMARY KEY (view_id, book_id)
        );

        CREATE TABLE IF NOT EXISTS snapshots (
            id               TEXT PRIMARY KEY,
            book_id          TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            type             TEXT NOT NULL CHECK(type IN ('full','incremental')),
            base_snapshot_id TEXT REFERENCES snapshots(id),
            data             TEXT NOT NULL,
            source           TEXT NOT NULL CHECK(source IN ('auto','manual')),
            created_at       TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS snapshot_tasks (
            id           TEXT PRIMARY KEY,
            book_id      TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            frequency    TEXT NOT NULL CHECK(frequency IN ('daily','weekly','monthly')),
            last_run_at  TEXT,
            is_active    INTEGER NOT NULL DEFAULT 1
        );

    "#);
    
    match &result {
        Ok(_) => println!("Migrations completed successfully"),
        Err(e) => println!("Migration failed: {:?}", e),
    }
    
    result?;

    // 字段迁移（列已存在时忽略错误）
    let _ = conn.execute("ALTER TABLE records ADD COLUMN name TEXT", []);

    Ok(())
}
