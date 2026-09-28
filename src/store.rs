use crate::model::{Block, BlockType, NewBlock};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;
use uuid::Uuid;

pub struct Store {
    conn: Connection,
}

type BlockRow = (
    uuid::Uuid,
    Option<uuid::Uuid>,
    f64,
    String,
    String,
    String,
    Option<String>,
);

impl Store {
    fn block_from_row(row: BlockRow) -> anyhow::Result<Block> {
        let (id, parent_id, position, props, r#type, created_at, updated_at) = row;

        // converts string from the props column of the blocks table
        // into a serde_json::Value JSON representation
        let props_val = serde_json::from_str::<serde_json::Value>(&props)?;

        // combines both type and props into a single serde_json::Value
        // reprensation that matches the BlockType serialization
        let type_val = json!({
            "type": r#type,
            "props": props_val,
        });

        Ok(Block {
            id,
            parent_id,
            position,
            r#type: serde_json::from_value::<BlockType>(type_val)?,
            created_at,
            updated_at,
        })
    }

    pub fn insert(&mut self, block: NewBlock) -> anyhow::Result<Block> {
        let block_type_value = serde_json::to_value(&block.r#type)?;

        let props = block_type_value.get("props").unwrap().to_string();
        let r#type = block_type_value.get("type").unwrap().as_str();

        self.conn.execute(
            "INSERT INTO blocks (id, parent_id, position, props, type)
            VALUES (?1, ?2, ?3, ?4, ?5);",
            params![block.id, block.parent_id, block.position, props, r#type],
        )?;

        let row = self.conn.query_row(
            "SELECT id, parent_id, position, props, type, created_at, updated_at FROM blocks WHERE id = ?1",
            params![block.id],
            |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?))
            })?;
        Self::block_from_row(row)
    }

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

    pub fn get(&self, id: Uuid) -> anyhow::Result<Option<Block>> {
        let row = self.conn.query_row(
            "SELECT id, parent_id, position, props, type, created_at, updated_at FROM blocks WHERE id = ?1",
            params![id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            }
        ).optional()?;

        Ok(match row {
            Some(r) => Some(Self::block_from_row(r)?),
            None => None,
        })
    }

    pub fn children_of(&self, parent_id: Uuid) -> anyhow::Result<Vec<Block>> {
        let mut blocks = vec![];

        let mut stmt = self.conn.prepare("SELECT id, parent_id, position, props, type, created_at, updated_at FROM blocks WHERE parent_id = ?1 ORDER BY position")?;
        let mut rows = stmt.query(params![parent_id])?;

        while let Some(row) = rows.next()? {
            let blockrow: BlockRow = (
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            );
            blocks.push(Self::block_from_row(blockrow)?);
        }
        Ok(blocks)
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

    #[test]
    fn insert_then_get() -> anyhow::Result<()> {
        let (id, parent_id, position, r#type) = (Uuid::new_v4(), None, 10.0, BlockType::Divider {});
        let new_block = NewBlock {
            id,
            parent_id,
            position,
            r#type: r#type.clone(),
        };
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;
        let inserted_block = store.insert(new_block)?;
        assert_eq!(id, inserted_block.id);
        assert_eq!(parent_id, inserted_block.parent_id);
        assert_eq!(position, inserted_block.position);
        assert_eq!(r#type, inserted_block.r#type);

        Ok(())
    }

    #[test]
    fn block_gets_found_by_id() -> anyhow::Result<()> {
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;
        let (id, parent_id, position, r#type) = (Uuid::new_v4(), None, 10.0, BlockType::Divider {});

        let new_block = NewBlock {
            id,
            parent_id,
            position,
            r#type: r#type,
        };
        let inserted_block = store.insert(new_block)?;

        let retreived_block = store.get(id)?.unwrap();

        assert_eq!(inserted_block, retreived_block);
        Ok(())
    }

    #[test]
    fn get_on_missing_id_returns_none() -> anyhow::Result<()> {
        let store = Store::open(":memory:")?;
        store.init_schema()?;
        let retreived_block = store.get(Uuid::new_v4())?;
        assert_eq!(retreived_block, None);

        Ok(())
    }

    #[test]
    fn children_of_block_found_by_parent_id() -> anyhow::Result<()> {
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;

        let parent_id = Uuid::new_v4();
        let _parent_block = store.insert(NewBlock {
            id: parent_id,
            parent_id: None,
            position: 10.0,
            r#type: BlockType::Page {
                title: "A Page".to_string(),
            },
        });
        let _child_block_1 = store.insert(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(parent_id),
            r#type: BlockType::Divider {},
            position: 11.0,
        });
        let _child_block_2 = store.insert(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(parent_id),
            r#type: BlockType::Divider {},
            position: 12.0,
        });
        let _child_block_3 = store.insert(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(parent_id),
            r#type: BlockType::Divider {},
            position: 13.0,
        });

        let children_blocks = store.children_of(parent_id)?;

        assert_eq!(children_blocks.len(), 3);

        Ok(())
    }
}
