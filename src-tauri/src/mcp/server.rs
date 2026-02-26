// src-tauri/src/mcp/server.rs
//! MCP HTTP 服务器 - 标准 MCP 2024-11-05 HTTP transport

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use tower_http::cors::{Any, CorsLayer};

use crate::mcp::{JsonRpcRequest, handle_request};

#[derive(Clone)]
struct McpState {
    db: Arc<Mutex<Connection>>,
}

pub async fn start_mcp_server(
    port: u16,
    db: Arc<Mutex<Connection>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = McpState { db };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/mcp", post(mcp_handler))
        .with_state(state)
        .layer(cors);

    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("MCP 服务器绑定端口 {} 失败: {}", port, e))?;

    println!("MCP server listening on http://{}/mcp", addr);
    println!(
        "  Claude Desktop 配置: {{\"mcpServers\":{{\"wealthy\":{{\"url\":\"http://127.0.0.1:{}/mcp\"}}}}}}",
        port
    );

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("MCP server error: {}", e))?;

    Ok(())
}

async fn mcp_handler(
    State(state): State<McpState>,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let req: JsonRpcRequest = match serde_json::from_value(payload) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": { "code": -32700, "message": format!("解析错误: {}", e) },
                    "id": null
                })),
            );
        }
    };

    let response = handle_request(&state.db, req);
    let json = serde_json::to_value(response).expect("序列化失败");
    (StatusCode::OK, Json(json))
}