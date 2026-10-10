use datablockrs::{
    model::{BlockType, NewBlock},
    store::Store,
};
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    let mut store = Store::open("db.sqlite").expect("should be able to open a sqlite database");
    store
        .init_schema()
        .expect("should be able to initialize a db schema");

    let root = store.append_child(NewBlock {
        id: Uuid::new_v4(),
        parent_id: None,
        r#type: BlockType::Database {
            title: "Database".to_string(),
        },
    })?;

    store.append_child(NewBlock {
        id: Uuid::new_v4(),
        parent_id: None,
        r#type: BlockType::Page {
            title: "Page One".to_string(),
            properties: vec![],
        },
    })?;

    store.append_child(NewBlock {
        id: Uuid::new_v4(),
        parent_id: None,
        r#type: BlockType::Page {
            title: "Page Two".to_string(),
            properties: vec![],
        },
    })?;
    store.append_child(NewBlock {
        id: Uuid::new_v4(),
        parent_id: None,
        r#type: BlockType::Page {
            title: "Page Three".to_string(),
            properties: vec![],
        },
    })?;

    let blocks = store.page_tree(root.id)?;

    println!("{}", serde_json::to_string_pretty(&blocks)?);

    Ok(())
}
