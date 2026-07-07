use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LsError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("DB: {0}")]
    Db(#[from] sqlx::Error),
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("Tauri: {0}")]
    Tauri(#[from] tauri::Error),
}

impl Serialize for LsError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type LsResult<T> = Result<T, LsError>;
