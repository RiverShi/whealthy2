use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use crate::error::{AppError, AppResult};

// ─── 数据结构 ─────────────────────────────────────────────────────────────────

/// 事件：轻量聚合单元，将语义相关的多条流水组合在一起
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub book_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

/// 流水记录：原子记账单元，可独立展示
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub id: String,
    pub book_id: String,
    pub event_id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub record_type: String, // income | expense | transfer
    pub amount: f64,
    pub category_id: Option<String>,
    pub from_account_id: Option<String>,
    pub to_account_id: Option<String>,
    pub tag_ids: Vec<String>,
    pub note: Option<String>,
    pub happened_at: String,
    pub created_at: String,
}

/// 事件 + 其下全部流水记录（用于详情接口）
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventWithRecords {
    #[serde(flatten)]
    pub event: Event,
    pub records: Vec<Record>,
}

// ─── 入参结构 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventParams {
    pub book_id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEventParams {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRecordParams {
    pub book_id: String,
    pub event_id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub record_type: String,
    pub amount: f64,
    pub category_id: Option<String>,
    pub from_account_id: Option<String>,
    pub to_account_id: Option<String>,
    pub tag_ids: Option<Vec<String>>,
    pub note: Option<String>,
    pub happened_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRecordParams {
    pub event_id: Option<Option<String>>,
    pub name: Option<Option<String>>,
    #[serde(rename = "type")]
    pub record_type: Option<String>,
    pub amount: Option<f64>,
    pub category_id: Option<Option<String>>,
    pub from_account_id: Option<Option<String>>,
    pub to_account_id: Option<Option<String>>,
    pub tag_ids: Option<Vec<String>>,
    pub note: Option<Option<String>>,
    pub happened_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordFilter {
    pub record_type: Option<String>,
    pub event_id: Option<String>,
    pub category_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

// ─── 内部工具 ─────────────────────────────────────────────────────────────────

fn load_record_tags(conn: &Connection, record_id: &str) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT tag_id FROM record_tags WHERE record_id = ?1"
    )?;
    let rows = stmt.query_map([record_id], |r| r.get(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn row_to_record(row: &rusqlite::Row) -> rusqlite::Result<Record> {
    Ok(Record {
        id: row.get(0)?,
        book_id: row.get(1)?,
        event_id: row.get(2)?,
        record_type: row.get(3)?,
        amount: row.get(4)?,
        category_id: row.get(5)?,
        from_account_id: row.get(6)?,
        to_account_id: row.get(7)?,
        note: row.get(8)?,
        happened_at: row.get(9)?,
        created_at: row.get(10)?,
        name: row.get(11)?,
        tag_ids: vec![],
    })
}

// ─── Event CRUD ───────────────────────────────────────────────────────────────

pub fn list_events(conn: &Connection, book_id: &str) -> AppResult<Vec<Event>> {
    let mut stmt = conn.prepare(
        "SELECT id, book_id, name, description, created_at
         FROM events WHERE book_id = ?1 ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map([book_id], |row| {
        Ok(Event {
            id: row.get(0)?,
            book_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn get_event(conn: &Connection, id: &str) -> AppResult<EventWithRecords> {
    let event = conn.query_row(
        "SELECT id, book_id, name, description, created_at FROM events WHERE id = ?1",
        [id],
        |row| Ok(Event {
            id: row.get(0)?,
            book_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            created_at: row.get(4)?,
        }),
    ).map_err(|_| AppError::NotFound(id.to_string()))?;

    let records = list_records_by_event(conn, id)?;
    Ok(EventWithRecords { event, records })
}

pub fn create_event(conn: &Connection, params: &CreateEventParams) -> AppResult<Event> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%d").to_string();
    conn.execute(
        "INSERT INTO events (id, book_id, name, description, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![&id, &params.book_id, &params.name, &params.description, &now],
    )?;
    Ok(Event {
        id,
        book_id: params.book_id.clone(),
        name: params.name.clone(),
        description: params.description.clone(),
        created_at: now,
    })
}

pub fn update_event(conn: &Connection, id: &str, params: &UpdateEventParams) -> AppResult<Event> {
    if let Some(name) = &params.name {
        conn.execute("UPDATE events SET name = ?1 WHERE id = ?2", rusqlite::params![name, id])?;
    }
    if let Some(desc) = &params.description {
        conn.execute("UPDATE events SET description = ?1 WHERE id = ?2", rusqlite::params![desc, id])?;
    }
    conn.query_row(
        "SELECT id, book_id, name, description, created_at FROM events WHERE id = ?1",
        [id],
        |row| Ok(Event {
            id: row.get(0)?,
            book_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            created_at: row.get(4)?,
        }),
    ).map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn delete_event(conn: &Connection, id: &str) -> AppResult<()> {
    // ON DELETE SET NULL 会自动解除 records.event_id 的关联
    let affected = conn.execute("DELETE FROM events WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(AppError::NotFound(id.to_string()));
    }
    Ok(())
}

// ─── Record CRUD ──────────────────────────────────────────────────────────────

pub fn list_records(conn: &Connection, book_id: &str, filter: Option<&RecordFilter>) -> AppResult<Vec<Record>> {
    let mut sql = String::from(
        "SELECT id, book_id, event_id, type, amount, category_id,
                from_account_id, to_account_id, note, happened_at, created_at, name
         FROM records WHERE book_id = ?1"
    );
    if let Some(f) = filter {
        if let Some(t) = &f.record_type {
            sql.push_str(&format!(" AND type = '{}'", t));
        }
        if let Some(eid) = &f.event_id {
            sql.push_str(&format!(" AND event_id = '{}'", eid));
        }
        if let Some(cid) = &f.category_id {
            sql.push_str(&format!(" AND category_id = '{}'", cid));
        }
        if let Some(from) = &f.from {
            sql.push_str(&format!(" AND happened_at >= '{}'", from));
        }
        if let Some(to) = &f.to {
            sql.push_str(&format!(" AND happened_at <= '{}'", to));
        }
    }
    sql.push_str(" ORDER BY happened_at DESC, created_at DESC");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([book_id], row_to_record)?;
    let mut records = rows.collect::<Result<Vec<_>, _>>()?;
    for r in &mut records {
        r.tag_ids = load_record_tags(conn, &r.id)?;
    }
    Ok(records)
}

fn list_records_by_event(conn: &Connection, event_id: &str) -> AppResult<Vec<Record>> {
    let mut stmt = conn.prepare(
        "SELECT id, book_id, event_id, type, amount, category_id,
                from_account_id, to_account_id, note, happened_at, created_at, name
         FROM records WHERE event_id = ?1
         ORDER BY happened_at ASC, created_at ASC"
    )?;
    let rows = stmt.query_map([event_id], row_to_record)?;
    let mut records = rows.collect::<Result<Vec<_>, _>>()?;
    for r in &mut records {
        r.tag_ids = load_record_tags(conn, &r.id)?;
    }
    Ok(records)
}

pub fn get_record(conn: &Connection, id: &str) -> AppResult<Record> {
    let mut record = conn.query_row(
        "SELECT id, book_id, event_id, type, amount, category_id,
                from_account_id, to_account_id, note, happened_at, created_at, name
         FROM records WHERE id = ?1",
        [id],
        row_to_record,
    ).map_err(|_| AppError::NotFound(id.to_string()))?;
    record.tag_ids = load_record_tags(conn, id)?;
    Ok(record)
}

pub fn create_record(conn: &Connection, params: &CreateRecordParams) -> AppResult<Record> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%d").to_string();
    conn.execute(
        "INSERT INTO records
            (id, book_id, event_id, name, type, amount, category_id,
             from_account_id, to_account_id, note, happened_at, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        rusqlite::params![
            &id,
            &params.book_id,
            &params.event_id,
            &params.name,
            &params.record_type,
            params.amount,
            &params.category_id,
            &params.from_account_id,
            &params.to_account_id,
            &params.note,
            &params.happened_at,
            &now,
        ],
    )?;

    let tag_ids = params.tag_ids.clone().unwrap_or_default();
    for tag_id in &tag_ids {
        conn.execute(
            "INSERT INTO record_tags (record_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![&id, tag_id],
        )?;
    }

    Ok(Record {
        id,
        book_id: params.book_id.clone(),
        event_id: params.event_id.clone(),
        name: params.name.clone(),
        record_type: params.record_type.clone(),
        amount: params.amount,
        category_id: params.category_id.clone(),
        from_account_id: params.from_account_id.clone(),
        to_account_id: params.to_account_id.clone(),
        tag_ids,
        note: params.note.clone(),
        happened_at: params.happened_at.clone(),
        created_at: now,
    })
}

pub fn update_record(conn: &Connection, id: &str, params: &UpdateRecordParams) -> AppResult<Record> {
    // 逐字段 patch
    if let Some(opt_eid) = &params.event_id {
        conn.execute("UPDATE records SET event_id = ?1 WHERE id = ?2", rusqlite::params![opt_eid, id])?;
    }
    if let Some(opt_name) = &params.name {
        conn.execute("UPDATE records SET name = ?1 WHERE id = ?2", rusqlite::params![opt_name, id])?;
    }
    if let Some(t) = &params.record_type {
        conn.execute("UPDATE records SET type = ?1 WHERE id = ?2", rusqlite::params![t, id])?;
    }
    if let Some(amt) = params.amount {
        conn.execute("UPDATE records SET amount = ?1 WHERE id = ?2", rusqlite::params![amt, id])?;
    }
    if let Some(opt_cid) = &params.category_id {
        conn.execute("UPDATE records SET category_id = ?1 WHERE id = ?2", rusqlite::params![opt_cid, id])?;
    }
    if let Some(opt_from) = &params.from_account_id {
        conn.execute("UPDATE records SET from_account_id = ?1 WHERE id = ?2", rusqlite::params![opt_from, id])?;
    }
    if let Some(opt_to) = &params.to_account_id {
        conn.execute("UPDATE records SET to_account_id = ?1 WHERE id = ?2", rusqlite::params![opt_to, id])?;
    }
    if let Some(opt_note) = &params.note {
        conn.execute("UPDATE records SET note = ?1 WHERE id = ?2", rusqlite::params![opt_note, id])?;
    }
    if let Some(hat) = &params.happened_at {
        conn.execute("UPDATE records SET happened_at = ?1 WHERE id = ?2", rusqlite::params![hat, id])?;
    }
    // 标签：全量替换
    if let Some(new_tags) = &params.tag_ids {
        conn.execute("DELETE FROM record_tags WHERE record_id = ?1", [id])?;
        for tag_id in new_tags {
            conn.execute(
                "INSERT INTO record_tags (record_id, tag_id) VALUES (?1, ?2)",
                rusqlite::params![id, tag_id],
            )?;
        }
    }

    get_record(conn, id)
}

pub fn delete_record(conn: &Connection, id: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM records WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(AppError::NotFound(id.to_string()));
    }
    Ok(())
}

// ─── 混合 Feed ────────────────────────────────────────────────────────────────

/// 事件摘要：包含旗下流水的聚合金额，用于混合 Feed 列表
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventSummary {
    pub id: String,
    pub book_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    /// 旗下流水收入总计
    pub total_income: f64,
    /// 旗下流水支出总计
    pub total_expense: f64,
    /// 旗下流水条数
    pub record_count: i64,
    /// 旗下最新流水的 happened_at（无流水时为 None，排序时回退到 created_at）
    pub latest_happened_at: Option<String>,
}

/// 混合 Feed 项：事件摘要 OR 独立流水记录（event_id IS NULL）
/// serde 使用相邻标签：{ "itemType": "event"|"record", ...fields }
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "itemType", rename_all = "camelCase")]
pub enum FeedItem {
    Event(EventSummary),
    Record(Record),
}

/// 混合 Feed 排序参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedSort {
    /// "date"（默认）或 "amount"
    pub sort_by: Option<String>,
    /// "desc"（默认）或 "asc"
    pub sort_order: Option<String>,
}

pub fn list_feed(conn: &Connection, book_id: &str, sort: Option<&FeedSort>) -> AppResult<Vec<FeedItem>> {
    // 1. 查所有事件，LEFT JOIN 聚合旗下金额
    let events: Vec<EventSummary> = {
        let mut stmt = conn.prepare(
            "SELECT e.id, e.book_id, e.name, e.description, e.created_at,
                    COALESCE(SUM(CASE WHEN r.type = 'income'  THEN r.amount ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN r.type = 'expense' THEN r.amount ELSE 0 END), 0),
                    COUNT(r.id),
                    MAX(r.happened_at)
             FROM events e
             LEFT JOIN records r ON r.event_id = e.id
             WHERE e.book_id = ?1
             GROUP BY e.id",
        )?;
        let rows = stmt.query_map([book_id], |row| {
            Ok(EventSummary {
                id: row.get(0)?,
                book_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                total_income: row.get(5)?,
                total_expense: row.get(6)?,
                record_count: row.get(7)?,
                latest_happened_at: row.get(8)?,
            })
        })?;
        rows.collect::<Result<_, _>>()?
    };

    // 2. 查无归属事件的独立流水（event_id IS NULL）
    let mut standalone: Vec<Record> = {
        let mut stmt = conn.prepare(
            "SELECT id, book_id, event_id, type, amount, category_id,
                    from_account_id, to_account_id, note, happened_at, created_at, name
             FROM records WHERE book_id = ?1 AND event_id IS NULL",
        )?;
        let rows = stmt.query_map([book_id], row_to_record)?;
        rows.collect::<Result<_, _>>()?
    };
    for r in &mut standalone {
        r.tag_ids = load_record_tags(conn, &r.id)?;
    }

    // 3. 合并为统一列表
    let mut items: Vec<FeedItem> = events
        .into_iter()
        .map(FeedItem::Event)
        .chain(standalone.into_iter().map(FeedItem::Record))
        .collect();

    // 4. 排序
    let sort_by = sort.and_then(|s| s.sort_by.as_deref()).unwrap_or("date");
    let desc = sort
        .and_then(|s| s.sort_order.as_deref())
        .map(|o| o != "asc")
        .unwrap_or(true);

    items.sort_by(|a, b| {
        if sort_by == "amount" {
            // 按流水总量排序（事件用收入+支出之和，单笔用金额）
            let ka = match a {
                FeedItem::Event(e) => e.total_income + e.total_expense,
                FeedItem::Record(r) => r.amount,
            };
            let kb = match b {
                FeedItem::Event(e) => e.total_income + e.total_expense,
                FeedItem::Record(r) => r.amount,
            };
            let ord = kb.partial_cmp(&ka).unwrap_or(std::cmp::Ordering::Equal);
            if desc { ord } else { ord.reverse() }
        } else {
            // 按日期排序（事件用最新流水日期，无则用创建日期）
            let da = match a {
                FeedItem::Event(e) => e.latest_happened_at.as_deref()
                    .unwrap_or(&e.created_at).to_string(),
                FeedItem::Record(r) => r.happened_at.clone(),
            };
            let db = match b {
                FeedItem::Event(e) => e.latest_happened_at.as_deref()
                    .unwrap_or(&e.created_at).to_string(),
                FeedItem::Record(r) => r.happened_at.clone(),
            };
            let ord = db.cmp(&da);
            if desc { ord } else { ord.reverse() }
        }
    });

    Ok(items)
}
