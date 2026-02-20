use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("参数错误: {0}")]
    InvalidInput(String),

    #[error("序列化错误: {0}")]
    Json(#[from] serde_json::Error),
}

// 让 AppError 可以被 Tauri command 返回
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
