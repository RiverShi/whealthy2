mod db;
mod error;
mod commands;
mod mcp;

use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use tauri::{State, Manager};
use error::{AppError, AppResult};
use commands::{books as cmd_books, entries as cmd_entries, categories as cmd_categories, records as cmd_records, snapshots as cmd_snapshots};
use cmd_books::Book;
use cmd_entries::{Entry, EntryAdjustment, CreateEntryParams, EntryFilter};
use cmd_categories::{Category, Tag};
use cmd_records::{Event, EventWithRecords, Record, CreateEventParams, UpdateEventParams, CreateRecordParams, UpdateRecordParams, RecordFilter, FeedItem, FeedSort};
use cmd_snapshots::{BookStats, Snapshot, SnapshotTask, SnapshotDiffEntry};

pub struct DbState(pub Arc<Mutex<Connection>>);

// ─── 账本 Commands ────────────────────────────────────────────────────────────

#[tauri::command]
fn list_books(state: State<DbState>) -> AppResult<Vec<Book>> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_books::list_books(&conn)
}

#[tauri::command]
fn create_book(state: State<DbState>, name: String) -> AppResult<Book> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_books::create_book(&conn, &name)
}

#[tauri::command]
fn update_book(state: State<DbState>, id: String, name: String) -> AppResult<Book> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_books::update_book(&conn, &id, &name)
}

#[tauri::command]
fn delete_book(state: State<DbState>, id: String) -> AppResult<()> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_books::delete_book(&conn, &id)
}

// ─── 条目 Commands ────────────────────────────────────────────────────────────

#[tauri::command]
fn list_entries(state: State<DbState>, book_id: String, filter: Option<EntryFilter>) -> AppResult<Vec<Entry>> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_entries::list_entries(&conn, &book_id, filter.as_ref())
}

#[tauri::command]
fn get_entry(state: State<DbState>, id: String) -> AppResult<Entry> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_entries::get_entry(&conn, &id)
}

#[tauri::command]
fn create_entry(state: State<DbState>, params: CreateEntryParams) -> AppResult<Entry> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_entries::create_entry(&conn, &params)
}

#[tauri::command]
fn update_entry(state: State<DbState>, id: String, params: CreateEntryParams) -> AppResult<Entry> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_entries::update_entry(&conn, &id, &params)
}

#[tauri::command]
fn delete_entry(state: State<DbState>, id: String) -> AppResult<()> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_entries::delete_entry(&conn, &id)
}

#[tauri::command]
fn adjust_entry_value(state: State<DbState>, id: String, new_value: f64, reason: Option<String>) -> AppResult<()> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_entries::adjust_entry_value(&conn, &id, new_value, reason.as_deref())
}

#[tauri::command]
fn list_entry_adjustments(state: State<DbState>, entry_id: String) -> AppResult<Vec<EntryAdjustment>> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_entries::list_entry_adjustments(&conn, &entry_id)
}

// ─── 分类 Commands ────────────────────────────────────────────────────────────

#[tauri::command]
fn list_categories(state: State<DbState>, domain: String, level: Option<u8>) -> AppResult<Vec<Category>> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_categories::list_categories(&conn, &domain, level)
}

#[tauri::command]
fn create_category(state: State<DbState>, domain: String, level: u8, name: String, parent_id: Option<String>, icon: Option<String>) -> AppResult<Category> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_categories::create_category(&conn, &domain, level, &name, parent_id.as_deref(), icon.as_deref())
}

#[tauri::command]
fn update_category(state: State<DbState>, id: String, name: Option<String>, icon: Option<String>, parent_id: Option<Option<String>>) -> AppResult<Category> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    let parent_id_ref = parent_id.as_ref().map(|opt| opt.as_deref());
    cmd_categories::update_category(&conn, &id, name.as_deref(), icon.as_deref(), parent_id_ref)
}

#[tauri::command]
fn delete_category(state: State<DbState>, id: String) -> AppResult<()> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_categories::delete_category(&conn, &id)
}

// ─── 标签 Commands ────────────────────────────────────────────────────────────

#[tauri::command]
fn list_tags(state: State<DbState>, domain: Option<String>) -> AppResult<Vec<Tag>> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_categories::list_tags(&conn, domain.as_deref())
}

#[tauri::command]
fn create_tag(state: State<DbState>, domain: String, name: String, color: Option<String>) -> AppResult<Tag> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_categories::create_tag(&conn, &domain, &name, color.as_deref())
}

#[tauri::command]
fn update_tag(state: State<DbState>, id: String, name: Option<String>, color: Option<String>) -> AppResult<Tag> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_categories::update_tag(&conn, &id, name.as_deref(), color.as_deref())
}

#[tauri::command]
fn delete_tag(state: State<DbState>, id: String) -> AppResult<()> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_categories::delete_tag(&conn, &id)
}

// ─── 事件 Commands ───────────────────────────────────────────────────────────

#[tauri::command]
fn list_events(state: State<DbState>, book_id: String) -> AppResult<Vec<Event>> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_records::list_events(&conn, &book_id)
}

#[tauri::command]
fn get_event(state: State<DbState>, id: String) -> AppResult<EventWithRecords> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_records::get_event(&conn, &id)
}

#[tauri::command]
fn create_event(state: State<DbState>, params: CreateEventParams) -> AppResult<Event> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_records::create_event(&conn, &params)
}

#[tauri::command]
fn update_event(state: State<DbState>, id: String, params: UpdateEventParams) -> AppResult<Event> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_records::update_event(&conn, &id, &params)
}

#[tauri::command]
fn delete_event(state: State<DbState>, id: String) -> AppResult<()> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_records::delete_event(&conn, &id)
}

// ─── 流水记录 Commands ────────────────────────────────────────────────────────

#[tauri::command]
fn list_records(state: State<DbState>, book_id: String, filter: Option<RecordFilter>) -> AppResult<Vec<Record>> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_records::list_records(&conn, &book_id, filter.as_ref())
}

#[tauri::command]
fn list_feed(state: State<DbState>, book_id: String, sort: Option<FeedSort>) -> AppResult<Vec<FeedItem>> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_records::list_feed(&conn, &book_id, sort.as_ref())
}

#[tauri::command]
fn get_record(state: State<DbState>, id: String) -> AppResult<Record> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_records::get_record(&conn, &id)
}

#[tauri::command]
fn create_record(state: State<DbState>, params: CreateRecordParams) -> AppResult<Record> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_records::create_record(&conn, &params)
}

#[tauri::command]
fn update_record(state: State<DbState>, id: String, params: UpdateRecordParams) -> AppResult<Record> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_records::update_record(&conn, &id, &params)
}

#[tauri::command]
fn delete_record(state: State<DbState>, id: String) -> AppResult<()> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_records::delete_record(&conn, &id)
}

// ─── 统计 Commands ────────────────────────────────────────────────────────────

#[tauri::command]
fn get_book_stats(state: State<DbState>, book_id: String, from: String, to: String) -> AppResult<BookStats> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_snapshots::get_book_stats(&conn, &book_id, &from, &to)
}

// ─── 快照 Commands ────────────────────────────────────────────────────────────

#[tauri::command]
fn list_snapshots(state: State<DbState>, book_id: String, from: Option<String>, to: Option<String>) -> AppResult<Vec<Snapshot>> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_snapshots::list_snapshots(&conn, &book_id, from.as_deref(), to.as_deref())
}

#[tauri::command]
fn get_snapshot(state: State<DbState>, id: String) -> AppResult<Snapshot> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_snapshots::get_snapshot(&conn, &id)
}

#[tauri::command]
fn create_snapshot(state: State<DbState>, book_id: String) -> AppResult<Snapshot> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_snapshots::create_snapshot(&conn, &book_id)
}

#[tauri::command]
fn diff_snapshots(state: State<DbState>, from_id: String, to_id: String) -> AppResult<Vec<SnapshotDiffEntry>> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_snapshots::diff_snapshots(&conn, &from_id, &to_id)
}

#[tauri::command]
fn list_snapshot_tasks(state: State<DbState>, book_id: Option<String>) -> AppResult<Vec<SnapshotTask>> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_snapshots::list_snapshot_tasks(&conn, book_id.as_deref())
}

#[tauri::command]
fn get_snapshot_task_for_book(state: State<DbState>, book_id: String) -> AppResult<Option<SnapshotTask>> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_snapshots::get_snapshot_task_for_book(&conn, &book_id)
}

#[tauri::command]
fn create_snapshot_task(state: State<DbState>, book_id: String, frequency: String) -> AppResult<SnapshotTask> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_snapshots::create_snapshot_task(&conn, &book_id, &frequency)
}

#[tauri::command]
fn update_snapshot_task(state: State<DbState>, id: String, frequency: Option<String>, is_active: Option<bool>) -> AppResult<SnapshotTask> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_snapshots::update_snapshot_task(&conn, &id, frequency.as_deref(), is_active)
}

#[tauri::command]
fn delete_snapshot_task(state: State<DbState>, id: String) -> AppResult<()> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_snapshots::delete_snapshot_task(&conn, &id)
}

#[tauri::command]
fn check_and_run_snapshot_tasks(state: State<DbState>) -> AppResult<()> {
    let conn = state.0.lock().map_err(|_| AppError::InvalidInput("锁失败".into()))?;
    cmd_snapshots::check_and_run_snapshot_tasks(&conn)
}

// ─── 应用入口 ─────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()
                .expect("无法获取 app data 目录");
            std::fs::create_dir_all(&app_dir).expect("无法创建 app data 目录");
            let db_path = app_dir.join("wealthy.db");
            let conn = Connection::open(&db_path).expect("无法打开数据库");
            db::initialize_database(&conn).expect("数据库初始化失败");
            // 共享同一个 Mutex<Connection>，Tauri 命令与 MCP 服务不再各自开连接
            let db_arc = Arc::new(Mutex::new(conn));
            app.manage(DbState(Arc::clone(&db_arc)));

            // 启动 MCP 服务，传入共享 DB 引用
            tauri::async_runtime::spawn(async move {
                if let Err(e) = crate::mcp::server::start_mcp_server(3030, db_arc).await {
                    eprintln!("MCP server error: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_books, create_book, update_book, delete_book,
            list_entries, get_entry, create_entry, update_entry, delete_entry,
            adjust_entry_value, list_entry_adjustments,
            list_categories, create_category, update_category, delete_category,
            list_tags, create_tag, update_tag, delete_tag,
            list_events, get_event, create_event, update_event, delete_event,
            list_records, get_record, create_record, update_record, delete_record,
            list_feed,
            get_book_stats,
            list_snapshots, get_snapshot, create_snapshot, diff_snapshots,
            list_snapshot_tasks, get_snapshot_task_for_book,
            create_snapshot_task, update_snapshot_task, delete_snapshot_task,
            check_and_run_snapshot_tasks,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用失败");
}
