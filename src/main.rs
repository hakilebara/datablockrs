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
    let parent_block = store.append_child(NewBlock {
        id: page_id,
        parent_id: None,
        r#type: BlockType::Page {
            title: "Main Page".to_string(),
        },
    })?;

    store.append_child(NewBlock {
        id: Uuid::new_v4(),
        parent_id: Some(page_id),
        r#type: BlockType::Divider {},
    })?;
    store.append_child(NewBlock {
        id: Uuid::new_v4(),
        parent_id: Some(page_id),
        r#type: BlockType::Divider {},
    })?;
    store.insert_at(
        NewBlock {
            id: Uuid::new_v4(),
            parent_id: Some(page_id),
            r#type: BlockType::Todo {
                checked: false,
                text: "Do it".to_string(),
            },
        },
        1,
    )?;

    let children_blocks = store.children_of(Some(page_id))?;

    println!(
        "parent block:\n{},\n\nchildren blocks:\n{}",
        serde_json::to_string_pretty(&parent_block)?,
        serde_json::to_string_pretty(&children_blocks)?
    );

    Ok(())
}
