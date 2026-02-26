use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, NaiveDate, Datelike};
use crate::error::{AppError, AppResult};

// ─── 统计数据 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategoryStat {
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BookStats {
    pub total_assets: f64,
    pub total_liabilities: f64,
    pub net_worth: f64,
    pub income: f64,
    pub expense: f64,
    /// 不计收支类（transfer/inflow/outflow）期间合计金额
    pub other: f64,
    pub income_by_category: Vec<CategoryStat>,
    pub expense_by_category: Vec<CategoryStat>,
}

pub fn get_book_stats(conn: &Connection, book_id: &str, from: &str, to: &str) -> AppResult<BookStats> {
    // 资产/负债净值（当前实时值）
    let total_assets: f64 = conn.query_row(
        "SELECT COALESCE(SUM(value), 0) FROM entries WHERE book_id = ?1 AND kind = 'asset' AND closed_at IS NULL",
        rusqlite::params![book_id],
        |r| r.get(0),
    )?;
    let total_liabilities: f64 = conn.query_row(
        "SELECT COALESCE(SUM(value), 0) FROM entries WHERE book_id = ?1 AND kind = 'liability' AND closed_at IS NULL",
        rusqlite::params![book_id],
        |r| r.get(0),
    )?;

    // 期间收入/支出合计
    let income: f64 = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM records WHERE book_id = ?1 AND type = 'income' AND happened_at >= ?2 AND happened_at <= ?3",
        rusqlite::params![book_id, from, to],
        |r| r.get(0),
    )?;
    let expense: f64 = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM records WHERE book_id = ?1 AND type = 'expense' AND happened_at >= ?2 AND happened_at <= ?3",
        rusqlite::params![book_id, from, to],
        |r| r.get(0),
    )?;
    let other: f64 = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM records WHERE book_id = ?1 AND type IN ('transfer', 'inflow', 'outflow') AND happened_at >= ?2 AND happened_at <= ?3",
        rusqlite::params![book_id, from, to],
        |r| r.get(0),
    )?;

    // 收入分类明细
    let mut stmt_income = conn.prepare(
        "SELECT r.category_id, c.name, COALESCE(SUM(r.amount), 0)
         FROM records r
         LEFT JOIN categories c ON c.id = r.category_id
         WHERE r.book_id = ?1 AND r.type = 'income' AND r.happened_at >= ?2 AND r.happened_at <= ?3
         GROUP BY r.category_id, c.name
         ORDER BY SUM(r.amount) DESC"
    )?;
    let income_by_category: Vec<CategoryStat> = stmt_income
        .query_map(rusqlite::params![book_id, from, to], |row| {
            Ok(CategoryStat {
                category_id: row.get(0)?,
                category_name: row.get(1)?,
                amount: row.get(2)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    // 支出分类明细
    let mut stmt_expense = conn.prepare(
        "SELECT r.category_id, c.name, COALESCE(SUM(r.amount), 0)
         FROM records r
         LEFT JOIN categories c ON c.id = r.category_id
         WHERE r.book_id = ?1 AND r.type = 'expense' AND r.happened_at >= ?2 AND r.happened_at <= ?3
         GROUP BY r.category_id, c.name
         ORDER BY SUM(r.amount) DESC"
    )?;
    let expense_by_category: Vec<CategoryStat> = stmt_expense
        .query_map(rusqlite::params![book_id, from, to], |row| {
            Ok(CategoryStat {
                category_id: row.get(0)?,
                category_name: row.get(1)?,
                amount: row.get(2)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(BookStats {
        total_assets,
        total_liabilities,
        net_worth: total_assets - total_liabilities,
        income,
        expense,
        other,
        income_by_category,
        expense_by_category,
    })
}

// ─── 快照数据结构 ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotEntryData {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub value: f64,
    pub category_l1_id: Option<String>,
    #[serde(default)]
    pub category_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotData {
    pub entries: Vec<SnapshotEntryData>,
    pub total_assets: f64,
    pub total_liabilities: f64,
    pub net_worth: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub id: String,
    pub book_id: String,
    pub source: String,
    pub net_worth: f64,
    pub total_assets: f64,
    pub total_liabilities: f64,
    pub data: SnapshotData,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotTask {
    pub id: String,
    pub book_id: String,
    pub frequency: String,
    pub last_run_at: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDiffEntry {
    pub entry_id: String,
    pub entry_name: String,
    pub kind: String,
    pub old_value: Option<f64>,
    pub new_value: Option<f64>,
    pub change: f64,
}

// ─── 内部工具 ─────────────────────────────────────────────────────────────────

fn take_snapshot_internal(conn: &Connection, book_id: &str, source: &str) -> AppResult<Snapshot> {
    let mut stmt = conn.prepare(
        "SELECT e.id, e.name, e.kind, e.value, e.category_l1_id, c.name
         FROM entries e
         LEFT JOIN categories c ON c.id = e.category_l1_id
         WHERE e.book_id = ?1 AND e.closed_at IS NULL
         ORDER BY e.kind DESC, COALESCE(c.name, ''), e.name"
    )?;
    let entries: Vec<SnapshotEntryData> = stmt
        .query_map(rusqlite::params![book_id], |row| {
            Ok(SnapshotEntryData {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: row.get(2)?,
                value: row.get(3)?,
                category_l1_id: row.get(4)?,
                category_name: row.get(5)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    let total_assets: f64 = entries.iter().filter(|e| e.kind == "asset").map(|e| e.value).sum();
    let total_liabilities: f64 = entries.iter().filter(|e| e.kind == "liability").map(|e| e.value).sum();
    let net_worth = total_assets - total_liabilities;

    let data = SnapshotData {
        entries,
        total_assets,
        total_liabilities,
        net_worth,
    };

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%d").to_string();
    let data_json = serde_json::to_string(&data)
        .map_err(|e| AppError::InvalidInput(e.to_string()))?;

    conn.execute(
        "INSERT INTO snapshots (id, book_id, type, data, source, created_at) VALUES (?1, ?2, 'full', ?3, ?4, ?5)",
        rusqlite::params![id, book_id, data_json, source, now],
    )?;

    Ok(Snapshot {
        id,
        book_id: book_id.to_string(),
        source: source.to_string(),
        net_worth,
        total_assets,
        total_liabilities,
        data,
        created_at: now,
    })
}

fn parse_snapshot_row(row: &rusqlite::Row) -> rusqlite::Result<(String, String, String, String, String)> {
    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
}

fn build_snapshot(id: String, book_id: String, source: String, data_str: String, created_at: String) -> AppResult<Snapshot> {
    let data: SnapshotData = serde_json::from_str(&data_str)
        .map_err(|e| AppError::InvalidInput(e.to_string()))?;
    let net_worth = data.net_worth;
    let total_assets = data.total_assets;
    let total_liabilities = data.total_liabilities;
    Ok(Snapshot { id, book_id, source, net_worth, total_assets, total_liabilities, data, created_at })
}

// ─── 快照 CRUD ───────────────────────────────────────────────────────────────

pub fn list_snapshots(conn: &Connection, book_id: &str, from: Option<&str>, to: Option<&str>) -> AppResult<Vec<Snapshot>> {
    let mut sql = String::from(
        "SELECT id, book_id, source, data, created_at FROM snapshots WHERE book_id = ?1"
    );
    if from.is_some() { sql.push_str(" AND created_at >= ?2"); }
    if to.is_some() {
        if from.is_some() {
            sql.push_str(" AND created_at <= ?3");
        } else {
            sql.push_str(" AND created_at <= ?2");
        }
    }
    sql.push_str(" ORDER BY created_at ASC");

    let mut stmt = conn.prepare(&sql)?;

    let rows: Vec<(String, String, String, String, String)> = match (from, to) {
        (Some(f), Some(t)) => stmt.query_map(rusqlite::params![book_id, f, t], parse_snapshot_row)?.collect::<Result<_, _>>()?,
        (Some(f), None)    => stmt.query_map(rusqlite::params![book_id, f], parse_snapshot_row)?.collect::<Result<_, _>>()?,
        (None, Some(t))    => stmt.query_map(rusqlite::params![book_id, t], parse_snapshot_row)?.collect::<Result<_, _>>()?,
        (None, None)       => stmt.query_map(rusqlite::params![book_id], parse_snapshot_row)?.collect::<Result<_, _>>()?,
    };

    rows.into_iter()
        .map(|(id, bid, src, data_str, cat)| build_snapshot(id, bid, src, data_str, cat))
        .collect()
}

pub fn get_snapshot(conn: &Connection, id: &str) -> AppResult<Snapshot> {
    let (sid, book_id, source, data_str, created_at) = conn.query_row(
        "SELECT id, book_id, source, data, created_at FROM snapshots WHERE id = ?1",
        rusqlite::params![id],
        parse_snapshot_row,
    ).map_err(|_| AppError::NotFound(id.to_string()))?;
    build_snapshot(sid, book_id, source, data_str, created_at)
}

pub fn create_snapshot(conn: &Connection, book_id: &str) -> AppResult<Snapshot> {
    take_snapshot_internal(conn, book_id, "manual")
}

pub fn diff_snapshots(conn: &Connection, from_id: &str, to_id: &str) -> AppResult<Vec<SnapshotDiffEntry>> {
    let from_snap = get_snapshot(conn, from_id)?;
    let to_snap = get_snapshot(conn, to_id)?;

    let from_map: std::collections::HashMap<&str, &SnapshotEntryData> =
        from_snap.data.entries.iter().map(|e| (e.id.as_str(), e)).collect();
    let to_map: std::collections::HashMap<&str, &SnapshotEntryData> =
        to_snap.data.entries.iter().map(|e| (e.id.as_str(), e)).collect();

    let mut result: Vec<SnapshotDiffEntry> = Vec::new();

    // 新增或变化的条目
    for entry in &to_snap.data.entries {
        let old_value = from_map.get(entry.id.as_str()).map(|e| e.value);
        let new_value = entry.value;
        let change = new_value - old_value.unwrap_or(0.0);
        if change.abs() > 0.001 || old_value.is_none() {
            result.push(SnapshotDiffEntry {
                entry_id: entry.id.clone(),
                entry_name: entry.name.clone(),
                kind: entry.kind.clone(),
                old_value,
                new_value: Some(new_value),
                change,
            });
        }
    }

    // 已删除的条目
    for entry in &from_snap.data.entries {
        if !to_map.contains_key(entry.id.as_str()) {
            result.push(SnapshotDiffEntry {
                entry_id: entry.id.clone(),
                entry_name: entry.name.clone(),
                kind: entry.kind.clone(),
                old_value: Some(entry.value),
                new_value: None,
                change: -entry.value,
            });
        }
    }

    // 按变化绝对值降序排列
    result.sort_by(|a, b| b.change.abs().partial_cmp(&a.change.abs()).unwrap_or(std::cmp::Ordering::Equal));
    Ok(result)
}

// ─── 快照任务 ─────────────────────────────────────────────────────────────────

pub fn list_snapshot_tasks(conn: &Connection, book_id: Option<&str>) -> AppResult<Vec<SnapshotTask>> {
    let rows: Vec<SnapshotTask> = if let Some(bid) = book_id {
        let mut stmt = conn.prepare(
            "SELECT id, book_id, frequency, last_run_at, is_active FROM snapshot_tasks WHERE book_id = ?1"
        )?;
        let mapped = stmt.query_map(rusqlite::params![bid], |row| {
            Ok(SnapshotTask {
                id: row.get(0)?,
                book_id: row.get(1)?,
                frequency: row.get(2)?,
                last_run_at: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
            })
        })?;
        mapped.collect::<Result<_, _>>()?
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, book_id, frequency, last_run_at, is_active FROM snapshot_tasks"
        )?;
        let mapped = stmt.query_map([], |row| {
            Ok(SnapshotTask {
                id: row.get(0)?,
                book_id: row.get(1)?,
                frequency: row.get(2)?,
                last_run_at: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
            })
        })?;
        mapped.collect::<Result<_, _>>()?
    };
    Ok(rows)
}

pub fn get_snapshot_task_for_book(conn: &Connection, book_id: &str) -> AppResult<Option<SnapshotTask>> {
    let mut stmt = conn.prepare(
        "SELECT id, book_id, frequency, last_run_at, is_active FROM snapshot_tasks WHERE book_id = ?1 LIMIT 1"
    )?;
    let mut rows = stmt.query_map(rusqlite::params![book_id], |row| {
        Ok(SnapshotTask {
            id: row.get(0)?,
            book_id: row.get(1)?,
            frequency: row.get(2)?,
            last_run_at: row.get(3)?,
            is_active: row.get::<_, i32>(4)? != 0,
        })
    })?;
    Ok(rows.next().transpose()?)
}

pub fn create_snapshot_task(conn: &Connection, book_id: &str, frequency: &str) -> AppResult<SnapshotTask> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO snapshot_tasks (id, book_id, frequency, is_active) VALUES (?1, ?2, ?3, 1)",
        rusqlite::params![id, book_id, frequency],
    )?;
    Ok(SnapshotTask {
        id,
        book_id: book_id.to_string(),
        frequency: frequency.to_string(),
        last_run_at: None,
        is_active: true,
    })
}

pub fn update_snapshot_task(conn: &Connection, id: &str, frequency: Option<&str>, is_active: Option<bool>) -> AppResult<SnapshotTask> {
    if let Some(f) = frequency {
        conn.execute("UPDATE snapshot_tasks SET frequency = ?1 WHERE id = ?2", rusqlite::params![f, id])?;
    }
    if let Some(active) = is_active {
        conn.execute("UPDATE snapshot_tasks SET is_active = ?1 WHERE id = ?2", rusqlite::params![active as i32, id])?;
    }
    conn.query_row(
        "SELECT id, book_id, frequency, last_run_at, is_active FROM snapshot_tasks WHERE id = ?1",
        rusqlite::params![id],
        |row| Ok(SnapshotTask {
            id: row.get(0)?,
            book_id: row.get(1)?,
            frequency: row.get(2)?,
            last_run_at: row.get(3)?,
            is_active: row.get::<_, i32>(4)? != 0,
        }),
    ).map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn delete_snapshot_task(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM snapshot_tasks WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}

pub fn check_and_run_snapshot_tasks(conn: &Connection) -> AppResult<()> {
    let today_str = Utc::now().format("%Y-%m-%d").to_string();
    let today = NaiveDate::parse_from_str(&today_str, "%Y-%m-%d")
        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());

    struct TaskRow {
        id: String,
        book_id: String,
        frequency: String,
        last_run_at: Option<String>,
    }

    let mut stmt = conn.prepare(
        "SELECT id, book_id, frequency, last_run_at FROM snapshot_tasks WHERE is_active = 1"
    )?;
    let tasks: Vec<TaskRow> = stmt.query_map([], |row| {
        Ok(TaskRow {
            id: row.get(0)?,
            book_id: row.get(1)?,
            frequency: row.get(2)?,
            last_run_at: row.get(3)?,
        })
    })?.collect::<Result<_, _>>()?;

    for task in tasks {
        let should_run = match task.last_run_at.as_deref() {
            None => true,
            Some(last) => {
                let last_date = NaiveDate::parse_from_str(last, "%Y-%m-%d")
                    .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
                match task.frequency.as_str() {
                    "daily"   => today > last_date,
                    "weekly"  => (today - last_date).num_days() >= 7,
                    "monthly" => today.year() > last_date.year()
                        || (today.year() == last_date.year() && today.month() > last_date.month()),
                    _ => false,
                }
            }
        };

        if should_run {
            take_snapshot_internal(conn, &task.book_id, "auto")?;
            conn.execute(
                "UPDATE snapshot_tasks SET last_run_at = ?1 WHERE id = ?2",
                rusqlite::params![today_str, task.id],
            )?;
        }
    }

    Ok(())
}
