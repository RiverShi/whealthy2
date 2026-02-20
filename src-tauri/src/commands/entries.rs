use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub book_id: String,
    pub name: String,
    pub kind: String,
    pub is_account: bool,
    pub valuation_type: String,
    pub value: f64,
    pub category_l1_id: Option<String>,
    pub category_l2_id: Option<String>,
    pub tag_ids: Vec<String>,
    pub extra: Option<serde_json::Value>,
    pub opened_at: String,
    pub closed_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EntryAdjustment {
    pub id: String,
    pub entry_id: String,
    pub old_value: f64,
    pub new_value: f64,
    pub reason: Option<String>,
    pub adjusted_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryFilter {
    pub kind: Option<String>,
    pub is_closed: Option<bool>,
    pub category_l1_id: Option<String>,
    pub tag_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntryParams {
    pub book_id: String,
    pub name: String,
    pub kind: String,
    pub is_account: bool,
    pub value: f64,
    pub valuation_type: String,
    pub category_l1_id: Option<String>,
    pub category_l2_id: Option<String>,
    pub tag_ids: Option<Vec<String>>,
    pub extra: Option<serde_json::Value>,
}

fn load_tags(conn: &Connection, entry_id: &str) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT tag_id FROM entry_tags WHERE entry_id = ?1"
    )?;
    let rows = stmt.query_map(rusqlite::params![entry_id], |r| r.get(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<Entry> {
    let id: String = row.get(0)?;
    let extra_str: Option<String> = row.get(7)?;
    let extra = extra_str
        .and_then(|s| serde_json::from_str(&s).ok());
    Ok(Entry {
        id: id.clone(),
        book_id: row.get(1)?,
        name: row.get(2)?,
        kind: row.get(3)?,
        is_account: row.get::<_, i32>(4)? != 0,
        valuation_type: row.get(5)?,
        value: row.get(6)?,
        category_l1_id: row.get(8)?,
        category_l2_id: row.get(9)?,
        tag_ids: vec![],
        extra,
        opened_at: row.get(10)?,
        closed_at: row.get(11)?,
    })
}

pub fn list_entries(conn: &Connection, book_id: &str, filter: Option<&EntryFilter>) -> AppResult<Vec<Entry>> {
    let mut sql = String::from(
        "SELECT id, book_id, name, kind, is_account, valuation_type, value, extra, category_l1_id, category_l2_id, opened_at, closed_at FROM entries WHERE book_id = ?1"
    );
    if let Some(f) = &filter {
        if let Some(kind) = &f.kind { sql.push_str(&format!(" AND kind = '{}'", kind)); }
        match f.is_closed {
            Some(true) => sql.push_str(" AND closed_at IS NOT NULL"),
            Some(false) => sql.push_str(" AND closed_at IS NULL"),
            None => {}
        }
        if let Some(cat) = &f.category_l1_id { sql.push_str(&format!(" AND category_l1_id = '{}'", cat)); }
    }
    sql.push_str(" ORDER BY opened_at DESC");

    let mut stmt = conn.prepare(&sql)?;
    let mut entries: Vec<Entry> = stmt.query_map(rusqlite::params![book_id], |row| {
        row_to_entry(row)
    })?.collect::<Result<_, _>>()?;

    for e in &mut entries {
        e.tag_ids = load_tags(conn, &e.id)?;
    }
    Ok(entries)
}

pub fn get_entry(conn: &Connection, id: &str) -> AppResult<Entry> {
    let mut entry = conn.query_row(
        "SELECT id, book_id, name, kind, is_account, valuation_type, value, extra, category_l1_id, category_l2_id, opened_at, closed_at FROM entries WHERE id = ?1",
        rusqlite::params![id],
        |row| row_to_entry(row),
    ).map_err(|_| AppError::NotFound(id.to_string()))?;
    entry.tag_ids = load_tags(conn, id)?;
    Ok(entry)
}

pub fn create_entry(conn: &Connection, params: &CreateEntryParams) -> AppResult<Entry> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let extra_str = params.extra.as_ref().map(|e| e.to_string());
    conn.execute(
        "INSERT INTO entries (id, book_id, name, kind, is_account, valuation_type, value, extra, category_l1_id, category_l2_id, opened_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            id, params.book_id, params.name, params.kind,
            params.is_account as i32, params.valuation_type, params.value,
            extra_str, params.category_l1_id, params.category_l2_id, now
        ],
    )?;
    if let Some(tags) = &params.tag_ids {
        for tag_id in tags {
            conn.execute(
                "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
                rusqlite::params![id, tag_id],
            )?;
        }
    }
    get_entry(conn, &id)
}

pub fn update_entry(conn: &Connection, id: &str, params: &CreateEntryParams) -> AppResult<Entry> {
    let extra_str = params.extra.as_ref().map(|e| e.to_string());
    conn.execute(
        "UPDATE entries SET name=?1, kind=?2, is_account=?3, valuation_type=?4, value=?5, extra=?6, category_l1_id=?7, category_l2_id=?8 WHERE id=?9",
        rusqlite::params![
            params.name, params.kind, params.is_account as i32,
            params.valuation_type, params.value, extra_str,
            params.category_l1_id, params.category_l2_id, id
        ],
    )?;
    // 重置标签
    conn.execute("DELETE FROM entry_tags WHERE entry_id = ?1", rusqlite::params![id])?;
    if let Some(tags) = &params.tag_ids {
        for tag_id in tags {
            conn.execute(
                "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
                rusqlite::params![id, tag_id],
            )?;
        }
    }
    get_entry(conn, id)
}

pub fn delete_entry(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM entries WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}

pub fn adjust_entry_value(conn: &Connection, id: &str, new_value: f64, reason: Option<&str>) -> AppResult<()> {
    let old_value: f64 = conn.query_row(
        "SELECT value FROM entries WHERE id = ?1",
        rusqlite::params![id],
        |r| r.get(0),
    ).map_err(|_| AppError::NotFound(id.to_string()))?;

    conn.execute("UPDATE entries SET value = ?1 WHERE id = ?2", rusqlite::params![new_value, id])?;

    let adj_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO entry_adjustments (id, entry_id, old_value, new_value, reason, adjusted_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![adj_id, id, old_value, new_value, reason, now],
    )?;
    Ok(())
}

pub fn list_entry_adjustments(conn: &Connection, entry_id: &str) -> AppResult<Vec<EntryAdjustment>> {
    let mut stmt = conn.prepare(
        "SELECT id, entry_id, old_value, new_value, reason, adjusted_at FROM entry_adjustments WHERE entry_id = ?1 ORDER BY adjusted_at DESC"
    )?;
    let rows = stmt.query_map(rusqlite::params![entry_id], |row| {
        Ok(EntryAdjustment {
            id: row.get(0)?,
            entry_id: row.get(1)?,
            old_value: row.get(2)?,
            new_value: row.get(3)?,
            reason: row.get(4)?,
            adjusted_at: row.get(5)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}
