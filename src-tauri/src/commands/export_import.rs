use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use std::collections::{HashMap, HashSet};
use crate::error::{AppError, AppResult};
use crate::commands::books::Book;
use crate::commands::categories::{Category, Tag};

// ─── 导出数据结构 ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookMeta {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryExport {
    pub id: String,
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
    pub adjustments: Vec<AdjustmentExport>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdjustmentExport {
    pub id: String,
    pub old_value: f64,
    pub new_value: f64,
    pub reason: Option<String>,
    pub adjusted_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventExport {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordExport {
    pub id: String,
    pub event_id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub record_type: String,
    pub amount: f64,
    pub category_id: Option<String>,
    pub from_account_id: Option<String>,
    pub to_account_id: Option<String>,
    pub tag_ids: Vec<String>,
    pub note: Option<String>,
    pub happened_at: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookExportData {
    pub version: String,
    pub exported_at: String,
    pub book: BookMeta,
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
    pub entries: Vec<EntryExport>,
    pub events: Vec<EventExport>,
    pub records: Vec<RecordExport>,
}

// ─── 导出 ─────────────────────────────────────────────────────────────────────

pub fn export_book(conn: &Connection, book_id: &str) -> AppResult<String> {
    // 1. 账本基本信息
    let book = crate::commands::books::get_book(conn, book_id)?;

    // 2. 收集本账本用到的分类 ID
    let mut cat_ids: HashSet<String> = HashSet::new();
    {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT category_l1_id, category_l2_id FROM entries WHERE book_id = ?1"
        )?;
        let rows = stmt.query_map(params![book_id], |row| {
            Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<String>>(1)?))
        })?;
        for row in rows {
            let (l1, l2) = row?;
            if let Some(id) = l1 { cat_ids.insert(id); }
            if let Some(id) = l2 { cat_ids.insert(id); }
        }
    }
    {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT category_id FROM records WHERE book_id = ?1 AND category_id IS NOT NULL"
        )?;
        let rows = stmt.query_map(params![book_id], |row| row.get::<_, String>(0))?;
        for row in rows { cat_ids.insert(row?); }
    }

    // 补全二级分类的父分类（一级）
    let cat_snapshot: Vec<String> = cat_ids.iter().cloned().collect();
    for cat_id in &cat_snapshot {
        if let Ok(parent_id) = conn.query_row::<Option<String>, _, _>(
            "SELECT parent_id FROM categories WHERE id = ?1",
            params![cat_id],
            |row| row.get(0),
        ) {
            if let Some(pid) = parent_id { cat_ids.insert(pid); }
        }
    }

    let mut categories: Vec<Category> = Vec::new();
    for cat_id in &cat_ids {
        if let Ok(cat) = conn.query_row(
            "SELECT id, domain, level, parent_id, name, icon FROM categories WHERE id = ?1",
            params![cat_id],
            |row| Ok(Category {
                id: row.get(0)?, domain: row.get(1)?,
                level: row.get::<_, u8>(2)?, parent_id: row.get(3)?,
                name: row.get(4)?, icon: row.get(5)?,
            }),
        ) { categories.push(cat); }
    }
    categories.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));

    // 3. 收集本账本用到的标签 ID
    let mut tag_ids: HashSet<String> = HashSet::new();
    {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT tag_id FROM entry_tags WHERE entry_id IN (SELECT id FROM entries WHERE book_id = ?1)"
        )?;
        let rows = stmt.query_map(params![book_id], |row| row.get::<_, String>(0))?;
        for row in rows { tag_ids.insert(row?); }
    }
    {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT tag_id FROM record_tags WHERE record_id IN (SELECT id FROM records WHERE book_id = ?1)"
        )?;
        let rows = stmt.query_map(params![book_id], |row| row.get::<_, String>(0))?;
        for row in rows { tag_ids.insert(row?); }
    }

    let mut tags: Vec<Tag> = Vec::new();
    for tag_id in &tag_ids {
        if let Ok(tag) = conn.query_row(
            "SELECT id, domain, name, color FROM tags WHERE id = ?1",
            params![tag_id],
            |row| Ok(Tag {
                id: row.get(0)?, domain: row.get(1)?, name: row.get(2)?, color: row.get(3)?,
            }),
        ) { tags.push(tag); }
    }
    tags.sort_by(|a, b| a.name.cmp(&b.name));

    // 4. 条目及调整记录
    let mut entries: Vec<EntryExport> = {
        let mut stmt = conn.prepare(
            "SELECT id, name, kind, is_account, valuation_type, value, extra, category_l1_id, category_l2_id, opened_at, closed_at FROM entries WHERE book_id = ?1 ORDER BY opened_at"
        )?;
        let rows = stmt.query_map(params![book_id], |row| {
            let extra_str: Option<String> = row.get(6)?;
            let extra = extra_str.and_then(|s| serde_json::from_str(&s).ok());
            Ok(EntryExport {
                id: row.get(0)?, name: row.get(1)?, kind: row.get(2)?,
                is_account: row.get::<_, i32>(3)? != 0,
                valuation_type: row.get(4)?, value: row.get(5)?,
                category_l1_id: row.get(7)?, category_l2_id: row.get(8)?,
                tag_ids: vec![], extra, opened_at: row.get(9)?,
                closed_at: row.get(10)?, adjustments: vec![],
            })
        })?;
        rows.collect::<Result<_, _>>()?
    };

    for entry in &mut entries {
        let mut stmt = conn.prepare("SELECT tag_id FROM entry_tags WHERE entry_id = ?1")?;
        entry.tag_ids = stmt.query_map(params![entry.id], |r| r.get(0))?
            .collect::<Result<_, _>>()?;

        let mut stmt = conn.prepare(
            "SELECT id, old_value, new_value, reason, adjusted_at FROM entry_adjustments WHERE entry_id = ?1 ORDER BY adjusted_at"
        )?;
        entry.adjustments = stmt.query_map(params![entry.id], |row| {
            Ok(AdjustmentExport {
                id: row.get(0)?, old_value: row.get(1)?,
                new_value: row.get(2)?, reason: row.get(3)?, adjusted_at: row.get(4)?,
            })
        })?.collect::<Result<_, _>>()?;
    }

    // 5. 事件
    let events: Vec<EventExport> = {
        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at FROM events WHERE book_id = ?1 ORDER BY created_at"
        )?;
        let rows = stmt.query_map(params![book_id], |row| {
            Ok(EventExport {
                id: row.get(0)?, name: row.get(1)?,
                description: row.get(2)?, created_at: row.get(3)?,
            })
        })?;
        rows.collect::<Result<_, _>>()?
    };

    // 6. 流水记录
    let mut records: Vec<RecordExport> = {
        let mut stmt = conn.prepare(
            "SELECT id, event_id, name, type, amount, category_id, from_account_id, to_account_id, note, happened_at, created_at FROM records WHERE book_id = ?1 ORDER BY happened_at"
        )?;
        let rows = stmt.query_map(params![book_id], |row| {
            Ok(RecordExport {
                id: row.get(0)?, event_id: row.get(1)?, name: row.get(2)?,
                record_type: row.get(3)?, amount: row.get(4)?,
                category_id: row.get(5)?, from_account_id: row.get(6)?,
                to_account_id: row.get(7)?, tag_ids: vec![],
                note: row.get(8)?, happened_at: row.get(9)?, created_at: row.get(10)?,
            })
        })?;
        rows.collect::<Result<_, _>>()?
    };

    for record in &mut records {
        let mut stmt = conn.prepare("SELECT tag_id FROM record_tags WHERE record_id = ?1")?;
        record.tag_ids = stmt.query_map(params![record.id], |r| r.get(0))?
            .collect::<Result<_, _>>()?;
    }

    let export_data = BookExportData {
        version: "1.0".to_string(),
        exported_at: Utc::now().to_rfc3339(),
        book: BookMeta { id: book.id, name: book.name, created_at: book.created_at },
        categories, tags, entries, events, records,
    };

    serde_json::to_string_pretty(&export_data)
        .map_err(|e| AppError::InvalidInput(format!("序列化失败: {}", e)))
}

// ─── 导入 ─────────────────────────────────────────────────────────────────────

pub fn import_book(conn: &Connection, json_data: &str, new_name: Option<&str>) -> AppResult<Book> {
    let export: BookExportData = serde_json::from_str(json_data)
        .map_err(|e| AppError::InvalidInput(format!("JSON 格式错误: {}", e)))?;

    // 版本检查
    if export.version != "1.0" {
        return Err(AppError::InvalidInput(format!("不支持的导出格式版本: {}", export.version)));
    }

    // 确定账本名称（若已存在则加后缀）
    let base_name = new_name.unwrap_or(&export.book.name);
    let mut final_name = base_name.to_string();
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM books WHERE name = ?1)",
        params![final_name],
        |row| row.get(0),
    )?;
    if exists {
        final_name = format!("{} (导入)", final_name);
    }

    let new_book_id = Uuid::new_v4().to_string();
    let today = Utc::now().format("%Y-%m-%d").to_string();
    conn.execute(
        "INSERT INTO books (id, name, created_at) VALUES (?1, ?2, ?3)",
        params![new_book_id, final_name, today],
    )?;

    // ── 分类：按 name+domain+level 复用已有，否则新建
    let mut cat_id_map: HashMap<String, String> = HashMap::new();
    let mut sorted_cats = export.categories.clone();
    sorted_cats.sort_by(|a, b| a.level.cmp(&b.level)); // L1 先于 L2

    for cat in &sorted_cats {
        let parent_new = cat.parent_id.as_ref().and_then(|pid| cat_id_map.get(pid)).cloned();

        let existing: Option<String> = if cat.level == 1 {
            conn.query_row(
                "SELECT id FROM categories WHERE domain = ?1 AND level = 1 AND name = ?2",
                params![cat.domain, cat.name],
                |row| row.get(0),
            ).ok()
        } else {
            conn.query_row(
                "SELECT id FROM categories WHERE domain = ?1 AND level = 2 AND name = ?2 AND parent_id IS ?3",
                params![cat.domain, cat.name, parent_new.as_deref()],
                |row| row.get(0),
            ).ok()
        };

        let new_id = if let Some(eid) = existing {
            eid
        } else {
            let nid = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO categories (id, domain, level, parent_id, name, icon) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![nid, cat.domain, cat.level, parent_new.as_deref(), cat.name, cat.icon.as_deref()],
            )?;
            nid
        };
        cat_id_map.insert(cat.id.clone(), new_id);
    }

    // ── 标签：按 name+domain 复用已有，否则新建
    let mut tag_id_map: HashMap<String, String> = HashMap::new();
    for tag in &export.tags {
        let existing: Option<String> = conn.query_row(
            "SELECT id FROM tags WHERE domain = ?1 AND name = ?2",
            params![tag.domain, tag.name],
            |row| row.get(0),
        ).ok();

        let new_id = if let Some(eid) = existing {
            eid
        } else {
            let nid = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO tags (id, domain, name, color) VALUES (?1, ?2, ?3, ?4)",
                params![nid, tag.domain, tag.name, tag.color.as_deref()],
            )?;
            nid
        };
        tag_id_map.insert(tag.id.clone(), new_id);
    }

    // ── 条目
    let mut entry_id_map: HashMap<String, String> = HashMap::new();
    for entry in &export.entries {
        let new_id = Uuid::new_v4().to_string();
        entry_id_map.insert(entry.id.clone(), new_id.clone());

        let cat_l1 = entry.category_l1_id.as_ref().and_then(|id| cat_id_map.get(id)).cloned();
        let cat_l2 = entry.category_l2_id.as_ref().and_then(|id| cat_id_map.get(id)).cloned();
        let extra_str = entry.extra.as_ref().map(|v| v.to_string());

        conn.execute(
            "INSERT INTO entries (id, book_id, name, kind, is_account, valuation_type, value, category_l1_id, category_l2_id, extra, opened_at, closed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                new_id, new_book_id, entry.name, entry.kind,
                entry.is_account as i32, entry.valuation_type, entry.value,
                cat_l1.as_deref(), cat_l2.as_deref(), extra_str.as_deref(),
                entry.opened_at, entry.closed_at.as_deref()
            ],
        )?;

        for tag_id in &entry.tag_ids {
            if let Some(ntid) = tag_id_map.get(tag_id) {
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
                    params![new_id, ntid],
                );
            }
        }

        for adj in &entry.adjustments {
            let adj_id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO entry_adjustments (id, entry_id, old_value, new_value, reason, adjusted_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![adj_id, new_id, adj.old_value, adj.new_value, adj.reason.as_deref(), adj.adjusted_at],
            )?;
        }
    }

    // ── 事件
    let mut event_id_map: HashMap<String, String> = HashMap::new();
    for event in &export.events {
        let new_id = Uuid::new_v4().to_string();
        event_id_map.insert(event.id.clone(), new_id.clone());
        conn.execute(
            "INSERT INTO events (id, book_id, name, description, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![new_id, new_book_id, event.name, event.description.as_deref(), event.created_at],
        )?;
    }

    // ── 流水记录
    for record in &export.records {
        let new_id = Uuid::new_v4().to_string();
        let new_event_id = record.event_id.as_ref().and_then(|id| event_id_map.get(id)).cloned();
        let cat_id = record.category_id.as_ref().and_then(|id| cat_id_map.get(id)).cloned();
        let from_acc = record.from_account_id.as_ref().and_then(|id| entry_id_map.get(id)).cloned();
        let to_acc = record.to_account_id.as_ref().and_then(|id| entry_id_map.get(id)).cloned();

        conn.execute(
            "INSERT INTO records (id, book_id, event_id, name, type, amount, category_id, from_account_id, to_account_id, note, happened_at, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                new_id, new_book_id, new_event_id.as_deref(),
                record.name.as_deref(), record.record_type, record.amount,
                cat_id.as_deref(), from_acc.as_deref(), to_acc.as_deref(),
                record.note.as_deref(), record.happened_at, record.created_at
            ],
        )?;

        for tag_id in &record.tag_ids {
            if let Some(ntid) = tag_id_map.get(tag_id) {
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO record_tags (record_id, tag_id) VALUES (?1, ?2)",
                    params![new_id, ntid],
                );
            }
        }
    }

    Ok(Book { id: new_book_id, name: final_name, created_at: today })
}
