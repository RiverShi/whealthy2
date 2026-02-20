use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub domain: String,
    pub level: u8,
    pub parent_id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub domain: String,
    pub name: String,
    pub color: Option<String>,
}

pub fn list_categories(conn: &Connection, domain: &str, level: Option<u8>) -> AppResult<Vec<Category>> {
    let sql = if let Some(l) = level {
        format!("SELECT id, domain, level, parent_id, name, icon FROM categories WHERE domain = ?1 AND level = {} ORDER BY name", l)
    } else {
        "SELECT id, domain, level, parent_id, name, icon FROM categories WHERE domain = ?1 ORDER BY level, name".to_string()
    };
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params![domain], |row| {
        Ok(Category {
            id: row.get(0)?,
            domain: row.get(1)?,
            level: row.get::<_, u8>(2)?,
            parent_id: row.get(3)?,
            name: row.get(4)?,
            icon: row.get(5)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn create_category(conn: &Connection, domain: &str, level: u8, name: &str, parent_id: Option<&str>, icon: Option<&str>) -> AppResult<Category> {
    if level == 2 && parent_id.is_none() {
        return Err(AppError::InvalidInput("二级分类必须指定父分类".to_string()));
    }
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO categories (id, domain, level, parent_id, name, icon) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, domain, level, parent_id, name, icon],
    )?;
    Ok(Category { id, domain: domain.to_string(), level, parent_id: parent_id.map(str::to_string), name: name.to_string(), icon: icon.map(str::to_string) })
}

pub fn update_category(conn: &Connection, id: &str, name: Option<&str>, icon: Option<&str>, parent_id: Option<Option<&str>>) -> AppResult<Category> {
    if let Some(n) = name {
        conn.execute("UPDATE categories SET name = ?1 WHERE id = ?2", rusqlite::params![n, id])?;
    }
    if let Some(ic) = icon {
        conn.execute("UPDATE categories SET icon = ?1 WHERE id = ?2", rusqlite::params![ic, id])?;
    }
    if let Some(pid) = parent_id {
        conn.execute("UPDATE categories SET parent_id = ?1 WHERE id = ?2", rusqlite::params![pid, id])?;
    }
    conn.query_row(
        "SELECT id, domain, level, parent_id, name, icon FROM categories WHERE id = ?1",
        rusqlite::params![id],
        |row| Ok(Category {
            id: row.get(0)?, domain: row.get(1)?, level: row.get::<_, u8>(2)?,
            parent_id: row.get(3)?, name: row.get(4)?, icon: row.get(5)?,
        }),
    ).map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn delete_category(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM categories WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}

pub fn list_tags(conn: &Connection, domain: Option<&str>) -> AppResult<Vec<Tag>> {
    let map_row = |row: &rusqlite::Row| -> rusqlite::Result<Tag> {
        Ok(Tag { id: row.get(0)?, domain: row.get(1)?, name: row.get(2)?, color: row.get(3)? })
    };
    if let Some(d) = domain {
        let mut stmt = conn.prepare(
            "SELECT id, domain, name, color FROM tags WHERE domain = ?1 ORDER BY name"
        )?;
        let rows = stmt.query_map(rusqlite::params![d], map_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, domain, name, color FROM tags ORDER BY domain, name"
        )?;
        let rows = stmt.query_map([], map_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
}

pub fn create_tag(conn: &Connection, domain: &str, name: &str, color: Option<&str>) -> AppResult<Tag> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO tags (id, domain, name, color) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, domain, name, color],
    )?;
    Ok(Tag { id, domain: domain.to_string(), name: name.to_string(), color: color.map(str::to_string) })
}

pub fn update_tag(conn: &Connection, id: &str, name: Option<&str>, color: Option<&str>) -> AppResult<Tag> {
    if let Some(n) = name {
        conn.execute("UPDATE tags SET name = ?1 WHERE id = ?2", rusqlite::params![n, id])?;
    }
    if let Some(c) = color {
        conn.execute("UPDATE tags SET color = ?1 WHERE id = ?2", rusqlite::params![c, id])?;
    }
    conn.query_row(
        "SELECT id, domain, name, color FROM tags WHERE id = ?1",
        rusqlite::params![id],
        |row| Ok(Tag { id: row.get(0)?, domain: row.get(1)?, name: row.get(2)?, color: row.get(3)? }),
    ).map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn delete_tag(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM tags WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}
