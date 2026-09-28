use datablockrs::{
    model::{Block, BlockType},
    store::Store,
};
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    let store = Store::open(&"db.sqlite")?;
    store.init_schema()?;

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
