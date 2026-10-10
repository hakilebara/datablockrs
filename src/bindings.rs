wit_bindgen::generate!({
    generate_all,
});

use uuid::Uuid;

use crate::{
    bindings::exports::datablockrs::blockstore::types::{self as wit},
    error::StoreError,
    model::{Block, BlockType, Language, NewBlock, PageProperty, PropertyValue},
    store::Store,
};
use std::sync::Mutex;

static STORE: Mutex<Option<Store>> = Mutex::new(None);

struct Component {}

impl wit::Guest for Component {
    fn init_schema() -> Result<(), wit::StoreError> {
        let store = Store::open(":memory:").map_err(StoreError::from)?;
        store.init_schema().map_err(StoreError::from)?;
        let mut guard = STORE.lock().unwrap();
        *guard = Some(store);
        Ok(())
    }

    fn get(id: wit::Uuid) -> Result<Option<wit::Block>, wit::StoreError> {
        let guard = STORE.lock().unwrap();
        let store = guard.as_ref().ok_or(StoreError::NotInitialized)?;
        let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
        Ok(store.get(id)?.map(|b| b.into()))
    }

    fn children_of(parent_id: Option<wit::Uuid>) -> Result<Vec<wit::Block>, wit::StoreError> {
        let guard = STORE.lock().unwrap();
        let store = guard.as_ref().ok_or(StoreError::NotInitialized)?;
        let parent_id = parent_id
            .map(|s| Uuid::parse_str(&s))
            .transpose()
            .map_err(|e| e.to_string())?;

        Ok(store
            .children_of(parent_id)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn append_child(block: wit::NewBlock) -> Result<wit::Block, wit::StoreError> {
        let mut guard = STORE.lock().unwrap();
        let store = guard.as_mut().ok_or(StoreError::NotInitialized)?;
        let block = NewBlock::try_from(block).map_err(|e| e.to_string())?;
        Ok(store.append_child(block)?.into())
    }

    fn insert_at(block: wit::NewBlock, index: u32) -> Result<wit::Block, wit::StoreError> {
        let mut guard = STORE.lock().unwrap();
        let store = guard.as_mut().ok_or(StoreError::NotInitialized)?;
        let block = NewBlock::try_from(block).map_err(|e| e.to_string())?;
        Ok(store.insert_at(block, index)?.into())
    }

    fn page_tree(page_id: wit::Uuid) -> Result<Vec<(u32, wit::Block)>, wit::StoreError> {
        let guard = STORE.lock().unwrap();
        let store = guard.as_ref().ok_or(StoreError::NotInitialized)?;
        let page_id = Uuid::parse_str(&page_id).map_err(|e| e.to_string())?;
        Ok(store
            .page_tree(page_id)?
            .into_iter()
            .map(|(depth, block)| (depth, block.into()))
            .collect())
    }

    fn delete_subtree(id: wit::Uuid) -> Result<(), wit::StoreError> {
        let mut guard = STORE.lock().unwrap();
        let store = guard.as_mut().ok_or(StoreError::NotInitialized)?;
        let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
        Ok(store.delete_subtree(id)?)
    }

    fn move_block(
        id: wit::Uuid,
        new_parent: Option<wit::Uuid>,
        index: u32,
    ) -> Result<wit::Block, wit::StoreError> {
        let mut guard = STORE.lock().unwrap();
        let store = guard.as_mut().ok_or(StoreError::NotInitialized)?;
        let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
        let new_parent = new_parent
            .map(|s| Uuid::parse_str(&s))
            .transpose()
            .map_err(|e| e.to_string())?;
        Ok(store.move_block(id, new_parent, index)?.into())
    }

    fn set_block(id: wit::Uuid, type_: wit::BlockType) -> Result<wit::Block, wit::StoreError> {
        let mut guard = STORE.lock().unwrap();
        let store = guard.as_mut().ok_or(StoreError::NotInitialized)?;
        let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
        Ok(store.set_block(id, type_.into())?.into())
    }
}

export!(Component);

impl From<wit::BlockType> for BlockType {
    fn from(b: wit::BlockType) -> Self {
        match b {
            wit::BlockType::Page(wit::PageProps { title, properties }) => BlockType::Page {
                title,
                properties: properties.into_iter().map(|p| p.into()).collect(),
            },
            wit::BlockType::Text(wit::TextProps { text }) => BlockType::Text { text },
            wit::BlockType::Heading1(wit::TextProps { text }) => BlockType::Heading1 { text },
            wit::BlockType::Heading2(wit::TextProps { text }) => BlockType::Heading2 { text },
            wit::BlockType::Heading3(wit::TextProps { text }) => BlockType::Heading3 { text },
            wit::BlockType::BulletedListItem(wit::TextProps { text }) => {
                BlockType::BulletedListItem { text }
            }
            wit::BlockType::NumberedListItem(wit::TextProps { text }) => {
                BlockType::NumberedListItem { text }
            }
            wit::BlockType::Todo(wit::TodoProps { checked, text }) => {
                BlockType::Todo { checked, text }
            }
            wit::BlockType::Code(wit::CodeProps { language, content }) => BlockType::Code {
                language: language.into(),
                content,
            },
            wit::BlockType::Quote(wit::TextProps { text }) => BlockType::Quote { text },
            wit::BlockType::Divider => BlockType::Divider {},
            wit::BlockType::Database(wit::DatabaseProps { title }) => BlockType::Database { title },
        }
    }
}

impl From<BlockType> for wit::BlockType {
    fn from(b: BlockType) -> Self {
        match b {
            BlockType::Page { title, properties } => wit::BlockType::Page(wit::PageProps {
                title,
                properties: properties.into_iter().map(|p| p.into()).collect(),
            }),
            BlockType::Text { text } => wit::BlockType::Text(wit::TextProps { text }),
            BlockType::Heading1 { text } => wit::BlockType::Heading1(wit::TextProps { text }),
            BlockType::Heading2 { text } => wit::BlockType::Heading2(wit::TextProps { text }),
            BlockType::Heading3 { text } => wit::BlockType::Heading3(wit::TextProps { text }),
            BlockType::BulletedListItem { text } => {
                wit::BlockType::BulletedListItem(wit::TextProps { text })
            }
            BlockType::NumberedListItem { text } => {
                wit::BlockType::NumberedListItem(wit::TextProps { text })
            }
            BlockType::Todo { checked, text } => {
                wit::BlockType::Todo(wit::TodoProps { checked, text })
            }
            BlockType::Quote { text } => wit::BlockType::Quote(wit::TextProps { text }),
            BlockType::Divider {} => wit::BlockType::Divider,
            BlockType::Code { language, content } => wit::BlockType::Code(wit::CodeProps {
                language: language.into(),
                content,
            }),
            BlockType::Database { title } => wit::BlockType::Database(wit::DatabaseProps { title }),
        }
    }
}

// inbound NewBlock
impl TryFrom<wit::NewBlock> for NewBlock {
    type Error = uuid::Error;

    fn try_from(b: wit::NewBlock) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&b.id)?;
        let parent_id: Option<Uuid> = b.parent_id.as_deref().map(Uuid::parse_str).transpose()?;
        Ok(NewBlock {
            id,
            parent_id,
            r#type: b.type_.into(),
        })
    }
}

// outbound Block (block read)
impl From<Block> for wit::Block {
    fn from(b: Block) -> Self {
        wit::Block {
            id: b.id.to_string(),
            parent_id: b.parent_id.as_ref().map(Uuid::to_string),
            type_: b.r#type.into(),
            position: b.position,
            created_at: b.created_at,
            updated_at: b.updated_at,
        }
    }
}

// outbound StoreError
impl From<StoreError> for wit::StoreError {
    fn from(e: StoreError) -> Self {
        e.to_string()
    }
}

impl From<Language> for wit::Language {
    fn from(l: Language) -> Self {
        match l {
            Language::HTML => wit::Language::Html,
            Language::JavaScript => wit::Language::Javascript,
            Language::Python => wit::Language::Python,
            Language::Markdown => wit::Language::Markdown,
        }
    }
}

impl From<wit::Language> for Language {
    fn from(l: wit::Language) -> Self {
        match l {
            wit::Language::Html => Language::HTML,
            wit::Language::Javascript => Language::JavaScript,
            wit::Language::Python => Language::Python,
            wit::Language::Markdown => Language::Markdown,
        }
    }
}

impl From<PageProperty> for wit::PageProperty {
    fn from(pp: PageProperty) -> Self {
        wit::PageProperty {
            id: pp.id,
            value: pp.value.into(),
        }
    }
}

impl From<wit::PageProperty> for PageProperty {
    fn from(pp: wit::PageProperty) -> Self {
        PageProperty {
            id: pp.id,
            value: pp.value.into(),
        }
    }
}

impl From<PropertyValue> for wit::PropertyValue {
    fn from(pv: PropertyValue) -> Self {
        match pv {
            PropertyValue::Text(s) => wit::PropertyValue::Text(s),
            PropertyValue::Number(f) => wit::PropertyValue::Number(f),
            PropertyValue::Checkbox(b) => wit::PropertyValue::Checkbox(b),
            PropertyValue::Select(s) => wit::PropertyValue::Select(s),
            PropertyValue::MultiSelect(vs) => wit::PropertyValue::MultiSelect(vs),
        }
    }
}

impl From<wit::PropertyValue> for PropertyValue {
    fn from(pv: wit::PropertyValue) -> Self {
        match pv {
            wit::PropertyValue::Text(s) => PropertyValue::Text(s),
            wit::PropertyValue::Number(f) => PropertyValue::Number(f),
            wit::PropertyValue::Checkbox(b) => PropertyValue::Checkbox(b),
            wit::PropertyValue::Select(s) => PropertyValue::Select(s),
            wit::PropertyValue::MultiSelect(vs) => PropertyValue::MultiSelect(vs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_type_round_trips_through_wit() {
        let variants = vec![
            BlockType::Page {
                title: "t".into(),
                properties: vec![],
            },
            BlockType::Text { text: "t".into() },
            BlockType::Heading1 { text: "t".into() },
            BlockType::Heading2 { text: "t".into() },
            BlockType::Heading3 { text: "t".into() },
            BlockType::BulletedListItem { text: "t".into() },
            BlockType::NumberedListItem { text: "t".into() },
            BlockType::Todo {
                checked: true,
                text: "t".into(),
            },
            BlockType::Quote { text: "t".into() },
            BlockType::Divider {},
            BlockType::Code {
                language: Language::Python,
                content: "t".into(),
            },
        ];

        for v in variants {
            let wit_v: wit::BlockType = v.clone().into();
            let back: BlockType = wit_v.into();
            assert_eq!(v, back);
        }
    }
}
