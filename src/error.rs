use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Block {id} not found")]
    NotFound { id: Uuid },
    #[error("Cycle detected")]
    CycleDetected,
    #[error("Unknown type {block_type} for block {id}")]
    UnknownBlockType { id: Uuid, block_type: String },
    #[error(transparent)]
    Sqlite(rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        match error {
            rusqlite::Error::SqliteFailure(e, _) => {
                // https://sqlite.org/rescode.html#constraint_trigger
                if e.extended_code == 1811 {
                    return Self::CycleDetected;
                }
                Self::Sqlite(error)
            }
            _ => Self::Sqlite(error),
        }
    }
}
