use datablockrs::model::{Block, BlockType};
use rusqlite::Connection;
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    let conn = Connection::open_in_memory()?;
    let mut stmt = conn.prepare(
        r#"
CREATE TABLE blocks (
    id TEXT PRIMARY KEY, 
    parent_id TEXT REFERENCES blocks(id) ON DELETE CASCADE,
    position REAL,
    props TEXT,
    type TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TEXT
)
    "#,
    )?;
    stmt.execute([])?;

    // let uuid = Uuid::new_v4();
    // stmt = conn.prepare("INSERT INTO blocks (id, parent_id, position, props, type, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)")?;
    // stmt.execute(params![uuid.to_string(), 10.0, ])?;

    let todo = Block {
        id: Uuid::new_v4(),
        parent_id: None,
        r#type: BlockType::Todo {
            checked: false,
            text: "Do it".to_string(),
        },
        position: 10.0,
        created_at: "2026-03-01T19:05:00.000Z".to_string(),
        updated_at: Some("2026-04-01T19:05:00.000Z".to_string()),
    };
    let divider = Block {
        id: Uuid::new_v4(),
        parent_id: None,
        r#type: BlockType::Divider {},
        position: 10.0,
        created_at: "2026-03-01T19:05:00.000Z".to_string(),
        updated_at: Some("2026-04-01T19:05:00.000Z".to_string()),
    };

    println!("{}", serde_json::to_string(&todo)?);
    println!("{}", serde_json::to_string(&divider)?);
    Ok(())
}
