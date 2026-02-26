use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

pub fn list_books(conn: &Connection) -> AppResult<Vec<Book>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, created_at FROM books ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Book {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn create_book(conn: &Connection, name: &str) -> AppResult<Book> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::InvalidInput("账本名称不能为空".to_string()));
    }
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM books WHERE name = ?1)",
        rusqlite::params![name],
        |row| row.get(0),
    )?;
    if exists {
        return Err(AppError::InvalidInput(format!("已存在同名账本「{}」", name)));
    }
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%d").to_string();
    conn.execute(
        "INSERT INTO books (id, name, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![id, name, now],
    )?;
    Ok(Book { id, name: name.to_string(), created_at: now })
}

pub fn update_book(conn: &Connection, id: &str, name: &str) -> AppResult<Book> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::InvalidInput("账本名称不能为空".to_string()));
    }
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM books WHERE name = ?1 AND id != ?2)",
        rusqlite::params![name, id],
        |row| row.get(0),
    )?;
    if exists {
        return Err(AppError::InvalidInput(format!("已存在同名账本「{}」", name)));
    }
    let affected = conn.execute(
        "UPDATE books SET name = ?1 WHERE id = ?2",
        rusqlite::params![name, id],
    )?;
    if affected == 0 { return Err(AppError::NotFound(id.to_string())); }
    get_book(conn, id)
}

pub fn delete_book(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM books WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}

pub fn get_book(conn: &Connection, id: &str) -> AppResult<Book> {
    conn.query_row(
        "SELECT id, name, created_at FROM books WHERE id = ?1",
        rusqlite::params![id],
        |row| Ok(Book {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        }),
    ).map_err(|_| AppError::NotFound(id.to_string()))
}
