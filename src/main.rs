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

    let page_id = Uuid::new_v4();
    store.insert(NewBlock {
        id: page_id,
        parent_id: None,
        r#type: BlockType::Page {
            title: "Main Page".to_string(),
        },
        position: 10.0,
    })?;

    store.insert(NewBlock {
        id: Uuid::new_v4(),
        parent_id: Some(page_id),
        r#type: BlockType::Divider {},
        position: 11.0,
    })?;
    store.insert(NewBlock {
        id: Uuid::new_v4(),
        parent_id: Some(page_id),
        r#type: BlockType::Divider {},
        position: 12.0,
    })?;
    store.insert(NewBlock {
        id: Uuid::new_v4(),
        parent_id: Some(page_id),
        r#type: BlockType::Todo {
            checked: false,
            text: "Do it".to_string(),
        },
        position: 13.0,
    })?;

    let children_blocks = store.children_of(page_id)?;

    println!("{:?}", serde_json::to_string_pretty(&children_blocks)?);

    Ok(())
}
