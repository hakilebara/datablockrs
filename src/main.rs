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
        r#type: BlockType::Page {
            title: "Root".to_string(),
        },
    })?;

    let first_child = store.append_child(NewBlock {
        id: Uuid::new_v4(),
        parent_id: Some(root.id),
        r#type: BlockType::Text {
            text: String::from("A | Root > A"),
        },
    })?;
    store.append_child(NewBlock {
        id: Uuid::new_v4(),
        parent_id: Some(root.id),
        r#type: BlockType::Text {
            text: String::from("B | Root > B"),
        },
    })?;
    store.append_child(NewBlock {
        id: Uuid::new_v4(),
        parent_id: Some(first_child.id),
        r#type: BlockType::Text {
            text: String::from("C | Root > A > C"),
        },
    })?;
    store.append_child(NewBlock {
        id: Uuid::new_v4(),
        parent_id: Some(root.id),
        r#type: BlockType::Text {
            text: String::from("D | Root > D"),
        },
    })?;

    let blocks = store.page_tree(root.id)?;

    println!("{}", serde_json::to_string_pretty(&blocks)?);

    Ok(())
}
