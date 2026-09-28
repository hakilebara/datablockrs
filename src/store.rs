use rusqlite::Connection;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        Ok(Store { conn })
    }

    pub fn init_schema(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            r#"
        BEGIN;
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
         END;
        COMMIT;
        "#,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::params;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn cascade_delete_removes_children() -> Result<(), rusqlite::Error> {
        let store = Store::open(":memory:")?;
        store.init_schema()?;
        let mut stmt = store.conn.prepare(
            r#"
            INSERT INTO blocks (id, parent_id, position, props, type)
            VALUES (?1, ?2, ?3, ?4, ?5);"#,
        )?;
        let parent_uuid = Uuid::new_v4();

        // parent block
        stmt.execute(params![
            parent_uuid,
            None::<Uuid>,
            10.0,
            "{\"text\": \"hello\"}",
            "todo"
        ])?;

        // child block
        stmt.execute(params![
            Uuid::new_v4(),
            parent_uuid,
            20.0,
            "{\"text\": \"hello\"}",
            "todo"
        ])?;

        let mut stmt = store.conn.prepare("SELECT COUNT(*) from blocks")?;
        let count: i64 = stmt.query_row([], |r| r.get(0))?;

        assert_eq!(count, 2);

        let mut stmt = store.conn.prepare("DELETE FROM blocks WHERE id = ?1")?;
        stmt.execute(params![parent_uuid])?;

        let mut stmt = store.conn.prepare("SELECT COUNT(*) from blocks")?;
        let count: i64 = stmt.query_row([], |r| r.get(0))?;

        assert_eq!(count, 0);

        Ok(())
    }
}
