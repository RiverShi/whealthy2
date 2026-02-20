use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: String,
    pub book_id: String,
    pub description: Option<String>,
    pub happened_at: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionItem {
    pub id: String,
    pub transaction_id: String,
    #[serde(rename = "type")]
    pub item_type: String, // income, expense, transfer
    pub amount: f64,
    pub category_id: Option<String>,
    pub from_account_id: Option<String>,
    pub to_account_id: Option<String>,
    pub tag_ids: Vec<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionWithItems {
    #[serde(flatten)]
    pub transaction: Transaction,
    pub items: Vec<TransactionItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransactionParams {
    pub book_id: String,
    pub description: Option<String>,
    pub happened_at: String,
    pub items: Vec<CreateTransactionItemParams>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransactionItemParams {
    #[serde(rename = "type")]
    pub item_type: String,
    pub amount: f64,
    pub category_id: Option<String>,
    pub from_account_id: Option<String>,
    pub to_account_id: Option<String>,
    pub tag_ids: Option<Vec<String>>,
    pub note: Option<String>,
}

pub fn list_transactions(conn: &Connection, book_id: &str) -> AppResult<Vec<Transaction>> {
    let mut stmt = conn.prepare(
        "SELECT id, book_id, description, happened_at, created_at 
         FROM transactions WHERE book_id = ?1 ORDER BY happened_at DESC, created_at DESC"
    )?;
    let rows = stmt.query_map([book_id], |row| {
        Ok(Transaction {
            id: row.get(0)?,
            book_id: row.get(1)?,
            description: row.get(2)?,
            happened_at: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn get_transaction(conn: &Connection, id: &str) -> AppResult<TransactionWithItems> {
    let transaction: Transaction = conn.query_row(
        "SELECT id, book_id, description, happened_at, created_at FROM transactions WHERE id = ?1",
        [id],
        |row| Ok(Transaction {
            id: row.get(0)?,
            book_id: row.get(1)?,
            description: row.get(2)?,
            happened_at: row.get(3)?,
            created_at: row.get(4)?,
        }),
    ).map_err(|_| AppError::NotFound(id.to_string()))?;

    let mut stmt = conn.prepare(
        "SELECT id, transaction_id, type, amount, category_id, from_account_id, to_account_id, note
         FROM transaction_items WHERE transaction_id = ?1"
    )?;
    let item_rows = stmt.query_map([id], |row| {
        Ok((
            row.get::<_, String>(0)?, // id
            row.get::<_, String>(1)?, // transaction_id
            row.get::<_, String>(2)?, // type
            row.get::<_, f64>(3)?, // amount
            row.get::<_, Option<String>>(4)?, // category_id
            row.get::<_, Option<String>>(5)?, // from_account_id
            row.get::<_, Option<String>>(6)?, // to_account_id
            row.get::<_, Option<String>>(7)?, // note
        ))
    })?;

    let mut items = Vec::new();
    for row in item_rows {
        let (iid, tid, itype, amt, cat, from_acc, to_acc, note) = row?;
        
        // 查询标签
        let mut tag_stmt = conn.prepare(
            "SELECT tag_id FROM transaction_item_tags WHERE item_id = ?1"
        )?;
        let tags: Vec<String> = tag_stmt.query_map([&iid], |r| r.get(0))?
            .collect::<Result<_, _>>()?;

        items.push(TransactionItem {
            id: iid,
            transaction_id: tid,
            item_type: itype,
            amount: amt,
            category_id: cat,
            from_account_id: from_acc,
            to_account_id: to_acc,
            tag_ids: tags,
            note,
        });
    }

    Ok(TransactionWithItems { transaction, items })
}

pub fn create_transaction(conn: &Connection, params: &CreateTransactionParams) -> AppResult<TransactionWithItems> {
    let transaction_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO transactions (id, book_id, description, happened_at, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            &transaction_id,
            &params.book_id,
            &params.description,
            &params.happened_at,
            &now
        ],
    )?;

    let mut items = Vec::new();
    for item_params in &params.items {
        let item_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO transaction_items (id, transaction_id, type, amount, category_id, from_account_id, to_account_id, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                &item_id,
                &transaction_id,
                &item_params.item_type,
                item_params.amount,
                &item_params.category_id,
                &item_params.from_account_id,
                &item_params.to_account_id,
                &item_params.note,
            ],
        )?;

        // 插入标签
        if let Some(tags) = &item_params.tag_ids {
            for tag_id in tags {
                conn.execute(
                    "INSERT INTO transaction_item_tags (item_id, tag_id) VALUES (?1, ?2)",
                    rusqlite::params![&item_id, tag_id],
                )?;
            }
        }

        items.push(TransactionItem {
            id: item_id,
            transaction_id: transaction_id.clone(),
            item_type: item_params.item_type.clone(),
            amount: item_params.amount,
            category_id: item_params.category_id.clone(),
            from_account_id: item_params.from_account_id.clone(),
            to_account_id: item_params.to_account_id.clone(),
            tag_ids: item_params.tag_ids.clone().unwrap_or_default(),
            note: item_params.note.clone(),
        });
    }

    Ok(TransactionWithItems {
        transaction: Transaction {
            id: transaction_id,
            book_id: params.book_id.clone(),
            description: params.description.clone(),
            happened_at: params.happened_at.clone(),
            created_at: now,
        },
        items,
    })
}

pub fn delete_transaction(conn: &Connection, id: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM transactions WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(AppError::NotFound(id.to_string()));
    }
    Ok(())
}
