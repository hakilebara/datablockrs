use crate::error::StoreError;
use crate::model::{Block, BlockType, NewBlock};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use uuid::Uuid;

pub struct Store {
    conn: Connection,
}

type BlockRow = (
    uuid::Uuid,
    Option<uuid::Uuid>,
    u32,
    String,
    String,
    String,
    Option<String>,
);

impl Store {
    fn block_from_row(row: BlockRow) -> Result<Block, StoreError> {
        let (id, parent_id, position, props, r#type, created_at, updated_at) = row;

        if !BlockType::is_valid_type_name(&r#type) {
            return Err(StoreError::UnknownBlockType {
                id,
                block_type: r#type,
            });
        }

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

    // TODO: use Rust newtype idiom to distinguish between prop:String and type:String
    fn prop_type_from_block(r#type: &BlockType) -> Result<(String, String), StoreError> {
        let block_type_value = serde_json::to_value(r#type)?;
        let props = block_type_value.get("props").unwrap().to_string();
        let r#type = block_type_value
            .get("type")
            .unwrap()
            .as_str()
            .expect("a block must have a type");
        Ok((r#type.to_owned(), props))
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
            position INTEGER NOT NULL,
            props TEXT,
            type TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_blocks_parent ON blocks (parent_id, position);

        CREATE TRIGGER IF NOT EXISTS tgr_updated_at
        AFTER UPDATE OF parent_id, props, type ON blocks
         BEGIN
            UPDATE blocks SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
         END;

        CREATE TRIGGER IF NOT EXISTS tgr_blocks_no_cycle_update
        BEFORE UPDATE OF parent_id ON blocks
        WHEN NEW.parent_id IS NOT NULL
        BEGIN
          SELECT RAISE(ABORT, 'cycle detected')
          WHERE EXISTS (
            WITH RECURSIVE ancestors(id) AS (
              SELECT NEW.parent_id
              UNION
              SELECT b.parent_id FROM blocks b
              JOIN ancestors a ON b.id = a.id
              WHERE b.parent_id IS NOT NULL
            )
            SELECT 1 FROM ancestors WHERE id = NEW.id
          );
        END;

        COMMIT;
        "#,
        )?;

        Ok(())
    }

    /// Gets a Block given a uuid
    pub fn get(&self, id: Uuid) -> Result<Option<Block>, StoreError> {
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

    pub fn children_of(&self, parent_id: Option<Uuid>) -> Result<Vec<Block>, StoreError> {
        let mut blocks = vec![];

        let mut stmt = self.conn.prepare("SELECT id, parent_id, position, props, type, created_at, updated_at FROM blocks WHERE parent_id IS ?1 ORDER BY position")?;
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

    pub fn append_child(&mut self, block: NewBlock) -> Result<Block, StoreError> {
        let (r#type, props) = Store::prop_type_from_block(&block.r#type)?;

        let tx = self.conn.transaction()?;

        // compute the block's position
        let max_sibling_position: Option<u32> = tx.query_row(
            "SELECT MAX(position) FROM blocks WHERE parent_id IS ?",
            params![block.parent_id],
            |row| row.get(0),
        )?;

        let position = if let Some(max) = max_sibling_position {
            max + 1
        } else {
            0
        };

        tx.execute(
            "INSERT INTO blocks (id, parent_id, position, props, type)
            VALUES (?1, ?2, ?3, ?4, ?5);",
            params![block.id, block.parent_id, position, props, r#type],
        )?;

        let row = tx.query_row(
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

        tx.commit()?;

        Self::block_from_row(row)
    }

    pub fn insert_at(&mut self, block: NewBlock, index: u32) -> Result<Block, StoreError> {
        let (r#type, props) = Store::prop_type_from_block(&block.r#type)?;

        let tx = self.conn.transaction()?;

        tx.execute(
            "UPDATE blocks SET position = position + 1 WHERE parent_id IS ?1 AND position >= ?2",
            params![block.parent_id, index],
        )?;

        tx.execute(
            "INSERT INTO blocks (id, parent_id, position, props, type)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            params![block.id, block.parent_id, index, props, r#type],
        )?;

        let row = tx.query_row(
            "SELECT id, parent_id, position, props, type, created_at, updated_at
            FROM blocks WHERE id = ?1",
            params![block.id],
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
            },
        )?;

        tx.commit()?;

        Self::block_from_row(row)
    }

    pub fn page_tree(&self, page_id: Uuid) -> Result<Vec<(u32, Block)>, StoreError> {
        let mut blocks = vec![];

        let mut stmt = self.conn.prepare(
            "
            WITH RECURSIVE tree(id, parent_id, position, props, type, created_at, updated_at, depth) AS (
              SELECT id, parent_id, position, props, type, created_at, updated_at, 0
              FROM blocks
              WHERE id = ?1

              UNION ALL

              SELECT b.id, b.parent_id, b.position, b.props, b.type, b.created_at, b.updated_at, t.depth + 1
              FROM tree t
              JOIN blocks b ON t.id = b.parent_id
              -- the DESC modifier will cause lower levels in the tree (with larger depth values)
              -- to be processed first by the recursive-select, resulting in a depth-first search
              ORDER BY 8 DESC, 3 ASC
            )
            SELECT id, parent_id, position, props, type, created_at, updated_at, depth FROM tree;
        ",
        )?;
        let mut rows = stmt.query(params![page_id])?;

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
            let depth: u32 = row.get(7)?;
            blocks.push((depth, Self::block_from_row(blockrow)?));
        }

        Ok(blocks)
    }

    pub fn delete_subtree(&mut self, id: Uuid) -> Result<(), StoreError> {
        match self
            .conn
            .execute("DELETE FROM blocks WHERE id = ?1", params![id])
        {
            Ok(n) => match n {
                0 => Err(StoreError::NotFound { id }),
                _ => Ok(()),
            },
            Err(e) => Err(StoreError::Sqlite(e)),
        }
    }

    pub fn move_block(
        &mut self,
        id: Uuid,
        new_parent: Option<Uuid>,
        index: u32,
    ) -> Result<Block, StoreError> {
        let tx = self.conn.transaction()?;

        // shift the new parent's siblings before re-parenting
        // so that same-parent moves also keep their order
        tx.execute(
            "
            UPDATE blocks SET position = position + 1
            WHERE parent_id IS ?1 AND position >= ?2
            ",
            params![new_parent, index],
        )?;

        tx.execute(
            "
            UPDATE blocks SET parent_id = ?1, position = ?2
            WHERE id = ?3
            ",
            params![new_parent, index, id],
        )?;

        let row: Option<BlockRow> = tx
            .query_row(
                "
            SELECT id, parent_id, position, props, type, created_at, updated_at
            FROM blocks WHERE id = ?1
            ",
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
                },
            )
            .optional()?;

        tx.commit()?;

        match row {
            Some(r) => Self::block_from_row(r),
            None => Err(StoreError::NotFound { id }),
        }
    }

    pub fn set_block(&mut self, id: Uuid, r#type: BlockType) -> Result<Block, StoreError> {
        let (r#type, props) = Store::prop_type_from_block(&r#type)?;
        let tx = self.conn.transaction()?;

        tx.execute(
            "UPDATE blocks SET type = ?1, props = ?2 WHERE id = ?3",
            params![r#type, props, id],
        )?;

        let row: Option<BlockRow> = tx
            .query_row(
                "
            SELECT id, parent_id, position, props, type, created_at, updated_at
            FROM blocks WHERE id = ?1
            ",
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
                },
            )
            .optional()?;

        tx.commit()?;

        match row {
            Some(r) => Self::block_from_row(r),
            None => Err(StoreError::NotFound { id }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

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
        let parent_id = Uuid::new_v4();

        // parent block
        stmt.execute(params![
            parent_id,
            None::<Uuid>,
            0,
            "{\"text\": \"hello\"}",
            "todo"
        ])?;

        // child block
        stmt.execute(params![
            Uuid::new_v4(),
            parent_id,
            1,
            "{\"text\": \"hello\"}",
            "todo"
        ])?;

        let mut stmt = store.conn.prepare("SELECT COUNT(*) from blocks")?;
        let count: i64 = stmt.query_row([], |r| r.get(0))?;

        assert_eq!(count, 2);

        let mut stmt = store.conn.prepare("DELETE FROM blocks WHERE id = ?1")?;
        stmt.execute(params![parent_id])?;

        let mut stmt = store.conn.prepare("SELECT COUNT(*) from blocks")?;
        let count: i64 = stmt.query_row([], |r| r.get(0))?;

        assert_eq!(count, 0);

        Ok(())
    }

    #[test]
    fn append_returns_block() -> anyhow::Result<()> {
        let (id, parent_id, r#type) = (Uuid::new_v4(), None, BlockType::Divider {});
        let new_block = NewBlock {
            id,
            parent_id,
            r#type: r#type.clone(),
        };
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;
        let inserted_block = store.append_child(new_block)?;
        assert_eq!(id, inserted_block.id);
        assert_eq!(parent_id, inserted_block.parent_id);
        assert_eq!(inserted_block.position, 0);
        assert_eq!(r#type, inserted_block.r#type);

        Ok(())
    }

    #[test]
    fn block_gets_found_by_id() -> anyhow::Result<()> {
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;
        let (id, parent_id, r#type) = (Uuid::new_v4(), None, BlockType::Divider {});

        let new_block = NewBlock {
            id,
            parent_id,
            r#type: r#type,
        };
        let inserted_block = store.append_child(new_block)?;

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
        store.append_child(NewBlock {
            id: parent_id,
            parent_id: None,
            r#type: BlockType::Page {
                title: "A Page".to_string(),
                properties: vec![],
            },
        })?;
        append_elements(&mut store, Some(parent_id), BlockType::Divider {}, 3)?;

        let children_blocks = store.children_of(Some(parent_id))?;

        assert_eq!(children_blocks.len(), 3);

        Ok(())
    }

    #[test]
    fn childrenof_returns_orphan_siblings() -> anyhow::Result<()> {
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;

        append_elements(&mut store, None, BlockType::Divider {}, 3)?;

        let children_blocks = store.children_of(None)?;

        assert_eq!(children_blocks.len(), 3);

        Ok(())
    }

    #[test]
    fn append_child_on_empty_parent() -> anyhow::Result<()> {
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;

        let block = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: None,
            r#type: BlockType::Divider {},
        })?;

        assert_eq!(block.position, 0);

        Ok(())
    }
    #[test]
    fn insert_block_at_empty_parent() -> anyhow::Result<()> {
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;

        let index = 0;

        let block = store.insert_at(
            NewBlock {
                id: Uuid::new_v4(),
                parent_id: None,
                r#type: BlockType::Divider {},
            },
            index,
        )?;

        assert_eq!(block.position, index);

        Ok(())
    }

    #[test]
    fn insert_in_the_middle() -> anyhow::Result<()> {
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;

        let parent_id = Uuid::new_v4();
        let block_0_id = Uuid::new_v4();
        let block_1_id = Uuid::new_v4();
        let middle_block_id = Uuid::new_v4();

        store.append_child(NewBlock {
            id: parent_id,
            parent_id: None,
            r#type: BlockType::Divider {},
        })?;

        let block_0 = store.append_child(NewBlock {
            id: block_0_id,
            parent_id: Some(parent_id),
            r#type: BlockType::Divider {},
        })?;
        assert_eq!(block_0.position, 0);

        let block_1 = store.append_child(NewBlock {
            id: block_1_id,
            parent_id: Some(parent_id),
            r#type: BlockType::Divider {},
        })?;
        assert_eq!(block_1.position, 1);

        let middle_block = store.insert_at(
            NewBlock {
                id: middle_block_id,
                parent_id: Some(parent_id),
                r#type: BlockType::Divider {},
            },
            1,
        )?;
        assert_eq!(middle_block.position, 1);

        let children = store.children_of(Some(parent_id))?;
        assert_eq!(children[0].id, block_0_id);
        assert_eq!(children[1].id, middle_block_id);
        assert_eq!(children[2].id, block_1_id);
        assert_eq!(children[0].position, 0);
        assert_eq!(children[1].position, 1);
        assert_eq!(children[2].position, 2);

        Ok(())
    }

    /// appends n elments of given type
    fn append_elements(
        store: &mut Store,
        parent_id: Option<Uuid>,
        r#type: BlockType,
        n: u32,
    ) -> anyhow::Result<()> {
        for _ in 0..n {
            store.append_child(NewBlock {
                id: Uuid::new_v4(),
                parent_id,
                r#type: r#type.clone(),
            })?;
        }
        Ok(())
    }

    #[test]
    fn can_return_page_tree() -> anyhow::Result<()> {
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;

        let parent_id = Uuid::new_v4();

        store.append_child(NewBlock {
            id: parent_id,
            parent_id: None,
            r#type: BlockType::Page {
                title: String::from("Main Page"),
                properties: vec![],
            },
        })?;

        store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(parent_id),
            r#type: BlockType::Divider {},
        })?;

        let sub_block_1 = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(parent_id),
            r#type: BlockType::Divider {},
        })?;

        // parent > sub_block_1 > sub_block_2
        let sub_block_2 = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(sub_block_1.id),
            r#type: BlockType::Divider {},
        })?;

        // parent > sub_block_1 > sub_block_2 > sub_block_3
        let sub_block_3 = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(sub_block_2.id),
            r#type: BlockType::Todo {
                checked: true,
                text: String::from("it is done"),
            },
        })?;

        let vec = store.page_tree(parent_id)?;
        assert_eq!(vec.len(), 5);

        let level_3_block: Vec<_> = vec.iter().filter(|(d, _)| *d == 3).collect();

        assert_eq!(level_3_block.len(), 1);
        let (_, block): &(u32, Block) = level_3_block[0];
        assert_eq!(*block, sub_block_3);

        Ok(())
    }

    /// Spec for `move_block(id, new_parent, index)`:
    /// - the block is re-parented to `new_parent` and lands at `index`
    ///   among the new siblings (shift semantics, like `insert_at`)
    /// - the old parent's remaining children keep their relative order
    /// - changing `parent_id` fires the trigger: `updated_at` is set on the
    ///   moved block, but NOT on the shifted siblings (position-only updates)
    #[test]
    fn move_block_reparents_at_index() -> anyhow::Result<()> {
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;

        let page_a = Uuid::new_v4();
        let page_b = Uuid::new_v4();

        store.append_child(NewBlock {
            id: page_a,
            parent_id: None,
            r#type: BlockType::Page {
                title: "Page A".to_string(),
                properties: vec![],
            },
        })?;
        store.append_child(NewBlock {
            id: page_b,
            parent_id: None,
            r#type: BlockType::Page {
                title: "Page B".to_string(),
                properties: vec![],
            },
        })?;

        let a0 = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(page_a),
            r#type: BlockType::Divider {},
        })?;
        let a1 = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(page_a),
            r#type: BlockType::Divider {},
        })?;
        let a2 = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(page_a),
            r#type: BlockType::Divider {},
        })?;
        let b0 = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(page_b),
            r#type: BlockType::Divider {},
        })?;
        let b1 = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(page_b),
            r#type: BlockType::Divider {},
        })?;

        let moved = store.move_block(a1.id, Some(page_b), 1)?;

        assert_eq!(moved.parent_id, Some(page_b));
        assert_eq!(moved.position, 1);
        assert!(
            moved.updated_at.is_some(),
            "re-parenting must bump updated_at"
        );

        let children_b = store.children_of(Some(page_b))?;
        assert_eq!(children_b.len(), 3);
        assert_eq!(children_b[0].id, b0.id);
        assert_eq!(children_b[1].id, a1.id);
        assert_eq!(children_b[2].id, b1.id);
        assert_eq!(children_b[1].position, 1);
        assert_eq!(children_b[2].position, 2);
        assert!(
            children_b[2].updated_at.is_none(),
            "position shift must not bump updated_at"
        );

        let children_a = store.children_of(Some(page_a))?;
        assert_eq!(children_a.len(), 2);
        assert_eq!(children_a[0].id, a0.id);
        assert_eq!(children_a[1].id, a2.id);

        Ok(())
    }

    /// CASCADE must remove the whole subtree: deleting a middle block
    /// removes its children AND grandchildren, leaving siblings intact.
    #[test]
    fn delete_subtree_removes_grandchildren() -> anyhow::Result<()> {
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;

        let root = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: None,
            r#type: BlockType::Page {
                title: "Root".to_string(),
                properties: vec![],
            },
        })?;

        let child = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(root.id),
            r#type: BlockType::Divider {},
        })?;
        let sibling = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(root.id),
            r#type: BlockType::Divider {},
        })?;
        let grandchild = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(child.id),
            r#type: BlockType::Divider {},
        })?;

        store.delete_subtree(child.id)?;

        assert!(store.get(grandchild.id)?.is_none());
        assert!(store.get(child.id)?.is_none());
        assert!(store.get(root.id)?.is_some());
        assert!(store.get(sibling.id)?.is_some());

        let children = store.children_of(Some(root.id))?;
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, sibling.id);

        Ok(())
    }

    /// The `tgr_blocks_no_cycle_update` trigger must refuse a move that
    /// would place a block under its own descendant.
    #[test]
    fn move_block_rejects_cycles() -> anyhow::Result<()> {
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;

        let parent = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: None,
            r#type: BlockType::Page {
                title: "Page".to_string(),
                properties: vec![],
            },
        })?;
        let child = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(parent.id),
            r#type: BlockType::Divider {},
        })?;
        let grandchild = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(child.id),
            r#type: BlockType::Divider {},
        })?;

        let result = store.move_block(parent.id, Some(grandchild.id), 0);
        assert!(result.is_err(), "moving under a descendant must fail");

        assert!(matches!(result, Err(StoreError::CycleDetected)));

        // the tree must be untouched after the failed move
        assert!(store.get(parent.id)?.is_some());
        assert_eq!(store.get(parent.id)?.unwrap().parent_id, None);
        let children = store.children_of(Some(child.id))?;
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, grandchild.id);

        Ok(())
    }

    /// The cycle guard lives in the schema, not in the Rust API:
    /// a raw SQL UPDATE attempting to create a cycle must be aborted too.
    #[test]
    fn cycle_guard_is_db_level() -> anyhow::Result<()> {
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;

        let parent = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: None,
            r#type: BlockType::Page {
                title: "Page".to_string(),
                properties: vec![],
            },
        })?;
        let child = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(parent.id),
            r#type: BlockType::Divider {},
        })?;

        let result = store.conn.execute(
            "UPDATE blocks SET parent_id = ?1 WHERE id = ?2",
            params![child.id, parent.id],
        );

        assert!(
            result.is_err(),
            "raw SQL cycle must be aborted by the trigger"
        );

        Ok(())
    }

    #[test]
    fn get_can_return_unknown_type_error() -> anyhow::Result<()> {
        let store = Store::open(":memory:")?;
        store.init_schema()?;

        let block_id = Uuid::new_v4();

        store.conn.execute(
            "INSERT INTO blocks (id, parent_id, position, props, type)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                block_id,
                None::<Uuid>,
                0,
                "{ text: 'Wrong type' }",
                String::from("carrot"),
            ],
        )?;

        let result = store.get(block_id);
        assert!(result.is_err());
        assert_matches!(result, Err(StoreError::UnknownBlockType { id, block_type })
            if block_type == "carrot" && id == block_id);

        Ok(())
    }

    #[test]
    fn move_block_can_return_not_found_error() -> anyhow::Result<()> {
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;

        let block_id = Uuid::new_v4();
        let result = store.move_block(block_id, None, 0);
        assert!(result.is_err());
        assert_matches!(
            result,
            Err(StoreError::NotFound { id }) if id == block_id
        );

        Ok(())
    }

    #[test]
    fn set_block_updates_a_block() -> anyhow::Result<()> {
        let mut store = Store::open(":memory:")?;
        store.init_schema()?;

        let block = store.append_child(NewBlock {
            id: Uuid::new_v4(),
            parent_id: None,
            r#type: BlockType::Todo {
                checked: true,
                text: String::from("checked"),
            },
        })?;

        assert!(block.updated_at.is_none());

        let updated_block = store.set_block(block.id, BlockType::Divider {})?;

        assert!(updated_block.updated_at.is_some());
        assert_matches!(updated_block.r#type, BlockType::Divider {});
        assert_eq!(updated_block.id, block.id);

        Ok(())
    }
}
