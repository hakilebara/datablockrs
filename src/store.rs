use rusqlite::Connection;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        Ok(Store {
            conn: Connection::open(path)?,
        })
    }

    pub fn init_schema(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            r#"
        PRAGMA foreign_keys ON;
        CREATE TABLE IF NOT EXISTS blocks (
            id TEXT PRIMARY KEY, 
            parent_id TEXT REFERENCES blocks(id) ON DELETE CASCADE,
            position REAL,
            props TEXT,
            type TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_blocks_parent ON blocks (parent_id, position);
        CREATE TRIGGER IF NOT EXISTS tgr_updated_at AFTER UPDATE ON blocks 
         BEGIN
            UPDATE blocks SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
         END
        "#,
        )?;

        Ok(())
    }
}
