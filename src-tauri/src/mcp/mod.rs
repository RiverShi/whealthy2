// src-tauri/src/mcp/mod.rs
//! Model Context Protocol 适配器层 - 标准 MCP 2024-11-05 实现

pub mod server;

use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::commands::{books, entries, categories, records, snapshots};

// ─── JSON-RPC 2.0 消息类型 ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: Option<String>,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

impl JsonRpcResponse {
    fn ok(id: Option<Value>, result: Value) -> Self {
        Self { jsonrpc: "2.0".into(), result: Some(result), error: None, id }
    }
    fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: None,
            error: Some(RpcError { code, message: message.into() }),
            id,
        }
    }
}

// ─── MCP 请求分发 ─────────────────────────────────────────────────────────────

pub fn handle_request(db: &Arc<Mutex<Connection>>, req: JsonRpcRequest) -> JsonRpcResponse {
    let id = req.id.clone();
    let params = req.params.unwrap_or(Value::Null);

    match req.method.as_str() {
        // ── MCP 生命周期 ────────────────────────────────────────────────────
        "initialize" => JsonRpcResponse::ok(id, json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "wealthy-mcp", "version": "0.1.0" }
        })),
        // 客户端通知，无需响应（null id 使客户端忽略此响应）
        "notifications/initialized" | "ping" => JsonRpcResponse::ok(None, Value::Null),
        // ── MCP 工具列表 ────────────────────────────────────────────────────
        "tools/list" => JsonRpcResponse::ok(id, json!({ "tools": tool_definitions() })),
        // ── MCP 工具调用 ────────────────────────────────────────────────────
        "tools/call" => {
            let tool_name = match params.get("name").and_then(|v| v.as_str()) {
                Some(n) => n.to_string(),
                None => return JsonRpcResponse::err(id, -32602, "缺少参数: name"),
            };
            let args = params.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));
            match call_tool(db, &tool_name, args) {
                Ok(content) => JsonRpcResponse::ok(id, json!({ "content": content })),
                Err(msg) => JsonRpcResponse::ok(id, json!({
                    "content": [{ "type": "text", "text": format!("错误: {}", msg) }],
                    "isError": true
                })),
            }
        }
        method => JsonRpcResponse::err(id, -32601, format!("不支持的方法: {}", method)),
    }
}

// ─── 工具定义 (JSON Schema) ───────────────────────────────────────────────────

fn tool_definitions() -> Value {
    json!([
        // ── 账本 ──────────────────────────────────────────────────────────
        { "name": "list_books", "description": "列出所有账本", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "create_book", "description": "创建新账本", "inputSchema": { "type": "object", "properties": { "name": { "type": "string", "description": "账本名称" } }, "required": ["name"] } },
        { "name": "update_book", "description": "修改账本名称", "inputSchema": { "type": "object", "properties": { "id": { "type": "string" }, "name": { "type": "string" } }, "required": ["id","name"] } },
        { "name": "delete_book", "description": "删除账本（同时删除其下所有数据）", "inputSchema": { "type": "object", "properties": { "id": { "type": "string" } }, "required": ["id"] } },

        // ── 资产/负债条目 ─────────────────────────────────────────────────
        { "name": "list_entries", "description": "列出账本中的资产/负债条目", "inputSchema": { "type": "object", "properties": { "bookId": { "type": "string", "description": "账本 ID" }, "filter": { "type": "object", "description": "可选过滤：{ kind: 'asset'|'liability', isClosed: bool }" } }, "required": ["bookId"] } },
        { "name": "create_entry", "description": "创建资产或负债条目", "inputSchema": { "type": "object", "properties": { "bookId": { "type": "string" }, "name": { "type": "string", "description": "条目名称，如「招商银行储蓄卡」" }, "kind": { "type": "string", "enum": ["asset","liability"] }, "isAccount": { "type": "boolean", "description": "是否是账户（可关联收支流水）" }, "value": { "type": "number", "description": "当前价值" }, "valuationType": { "type": "string", "enum": ["fixed","manual"] } }, "required": ["bookId","name","kind","isAccount","value","valuationType"] } },
        { "name": "adjust_entry_value", "description": "调整资产/负债条目的当前价值并记录原因", "inputSchema": { "type": "object", "properties": { "id": { "type": "string" }, "newValue": { "type": "number" }, "reason": { "type": "string" } }, "required": ["id","newValue"] } },
        { "name": "update_entry", "description": "更新条目信息（名称、分类等）", "inputSchema": { "type": "object", "properties": { "id": { "type": "string" }, "bookId": { "type": "string" }, "name": { "type": "string" }, "kind": { "type": "string", "enum": ["asset","liability"] }, "isAccount": { "type": "boolean" }, "value": { "type": "number" }, "valuationType": { "type": "string" }, "categoryL1Id": { "type": "string" }, "categoryL2Id": { "type": "string" } }, "required": ["id","bookId","name","kind","isAccount","value","valuationType"] } },
        { "name": "delete_entry", "description": "删除条目", "inputSchema": { "type": "object", "properties": { "id": { "type": "string" } }, "required": ["id"] } },

        // ── 流水记录 ───────────────────────────────────────────────────────
        { "name": "list_records", "description": "列出账本中的收支流水记录", "inputSchema": { "type": "object", "properties": { "bookId": { "type": "string" }, "filter": { "type": "object", "description": "可选：{ recordType, from, to (YYYY-MM-DD) }" } }, "required": ["bookId"] } },
        { "name": "create_record", "description": "创建一条收支或转账记录", "inputSchema": { "type": "object", "properties": { "bookId": { "type": "string" }, "type": { "type": "string", "enum": ["income","expense","transfer"] }, "amount": { "type": "number", "description": "金额（正数）" }, "happenedAt": { "type": "string", "description": "日期 YYYY-MM-DD" }, "categoryId": { "type": "string" }, "fromAccountId": { "type": "string" }, "toAccountId": { "type": "string" }, "note": { "type": "string" }, "eventId": { "type": "string" } }, "required": ["bookId","type","amount","happenedAt"] } },
        { "name": "update_record", "description": "更新流水记录（分类、备注等）", "inputSchema": { "type": "object", "properties": { "id": { "type": "string" }, "categoryId": { "type": "string" }, "note": { "type": "string" }, "amount": { "type": "number" }, "happenedAt": { "type": "string" }, "fromAccountId": { "type": "string" }, "toAccountId": { "type": "string" } }, "required": ["id"] } },
        { "name": "delete_record", "description": "删除流水记录", "inputSchema": { "type": "object", "properties": { "id": { "type": "string" } }, "required": ["id"] } },

        // ── 事件 ──────────────────────────────────────────────────────────
        { "name": "list_events", "description": "列出账本中的事件（多条流水的分组）", "inputSchema": { "type": "object", "properties": { "bookId": { "type": "string" } }, "required": ["bookId"] } },
        { "name": "create_event", "description": "创建事件（如「朋友聚餐 AA」）", "inputSchema": { "type": "object", "properties": { "bookId": { "type": "string" }, "name": { "type": "string" }, "description": { "type": "string" } }, "required": ["bookId","name"] } },
        { "name": "delete_event", "description": "删除事件（不级联删除其下流水）", "inputSchema": { "type": "object", "properties": { "id": { "type": "string" } }, "required": ["id"] } },

        // ── 分类与标签 ────────────────────────────────────────────────────
        { "name": "list_categories", "description": "列出分类（domain: asset/liability/income/expense）", "inputSchema": { "type": "object", "properties": { "domain": { "type": "string", "enum": ["asset","liability","income","expense"] }, "level": { "type": "number", "enum": [1,2] } }, "required": ["domain"] } },
        { "name": "create_category", "description": "创建分类（两级层级，level=1 为一级，level=2 需传 parentId）", "inputSchema": { "type": "object", "properties": { "domain": { "type": "string", "enum": ["asset","liability","income","expense"] }, "level": { "type": "number", "enum": [1,2] }, "name": { "type": "string" }, "parentId": { "type": "string", "description": "一级分类 ID（level=2 时必填）" }, "icon": { "type": "string" } }, "required": ["domain","level","name"] } },
        { "name": "delete_category", "description": "删除分类", "inputSchema": { "type": "object", "properties": { "id": { "type": "string" } }, "required": ["id"] } },
        { "name": "list_tags", "description": "列出标签", "inputSchema": { "type": "object", "properties": { "domain": { "type": "string", "enum": ["asset","liability","transaction"] } } } },
        { "name": "create_tag", "description": "创建标签", "inputSchema": { "type": "object", "properties": { "domain": { "type": "string", "enum": ["asset","liability","transaction"] }, "name": { "type": "string" }, "color": { "type": "string", "description": "十六进制颜色，如 #FF5733" } }, "required": ["domain","name"] } },
        { "name": "delete_tag", "description": "删除标签", "inputSchema": { "type": "object", "properties": { "id": { "type": "string" } }, "required": ["id"] } },

        // ── 统计与快照 ────────────────────────────────────────────────────
        { "name": "get_book_stats", "description": "获取账本统计（净值、收支汇总及分类明细）", "inputSchema": { "type": "object", "properties": { "bookId": { "type": "string" }, "from": { "type": "string", "description": "YYYY-MM-DD" }, "to": { "type": "string", "description": "YYYY-MM-DD" } }, "required": ["bookId","from","to"] } },
        { "name": "create_snapshot", "description": "为账本创建净值快照", "inputSchema": { "type": "object", "properties": { "bookId": { "type": "string" } }, "required": ["bookId"] } },
        { "name": "list_snapshots", "description": "列出历史净值快照", "inputSchema": { "type": "object", "properties": { "bookId": { "type": "string" }, "from": { "type": "string" }, "to": { "type": "string" } }, "required": ["bookId"] } }
    ])
}

// ─── 工具调用辅助 ─────────────────────────────────────────────────────────────

fn text_content(v: impl Serialize) -> Result<Vec<Value>, String> {
    let text = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
    Ok(vec![json!({ "type": "text", "text": text })])
}

// ─── 工具调用分发 ─────────────────────────────────────────────────────────────

fn call_tool(db: &Arc<Mutex<Connection>>, name: &str, args: Value) -> Result<Vec<Value>, String> {
    macro_rules! lock {
        () => {
            db.lock().map_err(|_| "数据库锁定失败".to_string())?
        };
    }
    match name {
        // ── 账本 ─────────────────────────────────────────────────────────
        "list_books" => {
            let conn = lock!();
            text_content(books::list_books(&conn).map_err(|e| e.to_string())?)
        }
        "create_book" => {
            let n = args.get("name").and_then(|v| v.as_str()).ok_or("缺少参数: name")?;
            let conn = lock!();
            text_content(books::create_book(&conn, n).map_err(|e| e.to_string())?)
        }
        "update_book" => {
            let id = args.get("id").and_then(|v| v.as_str()).ok_or("缺少参数: id")?;
            let n = args.get("name").and_then(|v| v.as_str()).ok_or("缺少参数: name")?;
            let conn = lock!();
            text_content(books::update_book(&conn, id, n).map_err(|e| e.to_string())?)
        }
        "delete_book" => {
            let id = args.get("id").and_then(|v| v.as_str()).ok_or("缺少参数: id")?;
            let conn = lock!();
            books::delete_book(&conn, id).map_err(|e| e.to_string())?;
            text_content(json!({ "success": true }))
        }
        // ── 资产/负债条目 ─────────────────────────────────────────────────
        "list_entries" => {
            let book_id = args.get("bookId").and_then(|v| v.as_str()).ok_or("缺少参数: bookId")?;
            let filter = args.get("filter")
                .and_then(|v| serde_json::from_value::<entries::EntryFilter>(v.clone()).ok());
            let conn = lock!();
            text_content(entries::list_entries(&conn, book_id, filter.as_ref()).map_err(|e| e.to_string())?)
        }
        "create_entry" => {
            let params: entries::CreateEntryParams = serde_json::from_value(args)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            let conn = lock!();
            text_content(entries::create_entry(&conn, &params).map_err(|e| e.to_string())?)
        }
        "adjust_entry_value" => {
            let id = args.get("id").and_then(|v| v.as_str()).ok_or("缺少参数: id")?;
            let new_value = args.get("newValue").and_then(|v| v.as_f64()).ok_or("缺少参数: newValue")?;
            let reason = args.get("reason").and_then(|v| v.as_str());
            let conn = lock!();
            entries::adjust_entry_value(&conn, id, new_value, reason).map_err(|e| e.to_string())?;
            text_content(json!({ "success": true }))
        }
        "update_entry" => {
            let id = args.get("id").and_then(|v| v.as_str()).ok_or("缺少参数: id")?.to_string();
            let params: entries::CreateEntryParams = serde_json::from_value(args)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            let conn = lock!();
            text_content(entries::update_entry(&conn, &id, &params).map_err(|e| e.to_string())?)
        }
        "delete_entry" => {
            let id = args.get("id").and_then(|v| v.as_str()).ok_or("缺少参数: id")?;
            let conn = lock!();
            entries::delete_entry(&conn, id).map_err(|e| e.to_string())?;
            text_content(json!({ "success": true }))
        }
        // ── 流水记录 ──────────────────────────────────────────────────────
        "list_records" => {
            let book_id = args.get("bookId").and_then(|v| v.as_str()).ok_or("缺少参数: bookId")?;
            let filter = args.get("filter")
                .and_then(|v| serde_json::from_value::<records::RecordFilter>(v.clone()).ok());
            let conn = lock!();
            text_content(records::list_records(&conn, book_id, filter.as_ref()).map_err(|e| e.to_string())?)
        }
        "create_record" => {
            let params: records::CreateRecordParams = serde_json::from_value(args)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            let conn = lock!();
            text_content(records::create_record(&conn, &params).map_err(|e| e.to_string())?)
        }
        "delete_record" => {
            let id = args.get("id").and_then(|v| v.as_str()).ok_or("缺少参数: id")?;
            let conn = lock!();
            records::delete_record(&conn, id).map_err(|e| e.to_string())?;
            text_content(json!({ "success": true }))
        }
        "update_record" => {
            let id = args.get("id").and_then(|v| v.as_str()).ok_or("缺少参数: id")?.to_string();
            let params: records::UpdateRecordParams = serde_json::from_value(args)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            let conn = lock!();
            text_content(records::update_record(&conn, &id, &params).map_err(|e| e.to_string())?)
        }
        // ── 事件 ─────────────────────────────────────────────────────────
        "list_events" => {
            let book_id = args.get("bookId").and_then(|v| v.as_str()).ok_or("缺少参数: bookId")?;
            let conn = lock!();
            text_content(records::list_events(&conn, book_id).map_err(|e| e.to_string())?)
        }
        "create_event" => {
            let params: records::CreateEventParams = serde_json::from_value(args)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            let conn = lock!();
            text_content(records::create_event(&conn, &params).map_err(|e| e.to_string())?)
        }
        "delete_event" => {
            let id = args.get("id").and_then(|v| v.as_str()).ok_or("缺少参数: id")?;
            let conn = lock!();
            records::delete_event(&conn, id).map_err(|e| e.to_string())?;
            text_content(json!({ "success": true }))
        }
        // ── 分类与标签 ────────────────────────────────────────────────────
        "list_categories" => {
            let domain = args.get("domain").and_then(|v| v.as_str()).unwrap_or("expense");
            let level = args.get("level").and_then(|v| v.as_u64()).map(|v| v as u8);
            let conn = lock!();
            text_content(categories::list_categories(&conn, domain, level).map_err(|e| e.to_string())?)
        }
        "create_category" => {
            let domain = args.get("domain").and_then(|v| v.as_str()).ok_or("缺少参数: domain")?;
            let level = args.get("level").and_then(|v| v.as_u64()).ok_or("缺少参数: level")? as u8;
            let name = args.get("name").and_then(|v| v.as_str()).ok_or("缺少参数: name")?;
            let parent_id = args.get("parentId").and_then(|v| v.as_str());
            let icon = args.get("icon").and_then(|v| v.as_str());
            let conn = lock!();
            text_content(categories::create_category(&conn, domain, level, name, parent_id, icon).map_err(|e| e.to_string())?)
        }
        "delete_category" => {
            let id = args.get("id").and_then(|v| v.as_str()).ok_or("缺少参数: id")?;
            let conn = lock!();
            categories::delete_category(&conn, id).map_err(|e| e.to_string())?;
            text_content(json!({ "success": true }))
        }
        "list_tags" => {
            let domain = args.get("domain").and_then(|v| v.as_str());
            let conn = lock!();
            text_content(categories::list_tags(&conn, domain).map_err(|e| e.to_string())?)
        }
        "create_tag" => {
            let domain = args.get("domain").and_then(|v| v.as_str()).ok_or("缺少参数: domain")?;
            let name = args.get("name").and_then(|v| v.as_str()).ok_or("缺少参数: name")?;
            let color = args.get("color").and_then(|v| v.as_str());
            let conn = lock!();
            text_content(categories::create_tag(&conn, domain, name, color).map_err(|e| e.to_string())?)
        }
        "delete_tag" => {
            let id = args.get("id").and_then(|v| v.as_str()).ok_or("缺少参数: id")?;
            let conn = lock!();
            categories::delete_tag(&conn, id).map_err(|e| e.to_string())?;
            text_content(json!({ "success": true }))
        }
        // ── 统计与快照 ────────────────────────────────────────────────────
        "get_book_stats" => {
            let book_id = args.get("bookId").and_then(|v| v.as_str()).ok_or("缺少参数: bookId")?;
            let from = args.get("from").and_then(|v| v.as_str()).ok_or("缺少参数: from")?;
            let to = args.get("to").and_then(|v| v.as_str()).ok_or("缺少参数: to")?;
            let conn = lock!();
            text_content(snapshots::get_book_stats(&conn, book_id, from, to).map_err(|e| e.to_string())?)
        }
        "list_snapshots" => {
            let book_id = args.get("bookId").and_then(|v| v.as_str()).ok_or("缺少参数: bookId")?;
            let from = args.get("from").and_then(|v| v.as_str());
            let to = args.get("to").and_then(|v| v.as_str());
            let conn = lock!();
            text_content(snapshots::list_snapshots(&conn, book_id, from, to).map_err(|e| e.to_string())?)
        }
        "create_snapshot" => {
            let book_id = args.get("bookId").and_then(|v| v.as_str()).ok_or("缺少参数: bookId")?;
            let conn = lock!();
            text_content(snapshots::create_snapshot(&conn, book_id).map_err(|e| e.to_string())?)
        }
        unknown => Err(format!("未知工具: {}", unknown)),
    }
}
