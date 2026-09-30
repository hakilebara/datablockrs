use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Language {
    JavaScript,
    HTML,
    Python,
    Markdown,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(tag = "type", content = "props", rename_all = "snake_case")]
pub enum BlockType {
    Page { title: String },
    Text { text: String },
    Heading1 { text: String },
    Heading2 { text: String },
    Heading3 { text: String },
    BulletedListItem { text: String },
    NumberedListItem { text: String },
    Todo { checked: bool, text: String },
    Quote { text: String },
    Divider {},
    Code { language: Language, content: String },
}

impl BlockType {
    pub fn type_name(&self) -> &'static str {
        match self {
            BlockType::Page { .. } => "page",
            BlockType::Text { .. } => "text",
            BlockType::Heading1 { .. } => "heading1",
            BlockType::Heading2 { .. } => "heading2",
            BlockType::Heading3 { .. } => "heading3",
            BlockType::BulletedListItem { .. } => "bulleted_list_item",
            BlockType::NumberedListItem { .. } => "numbered_list_item",
            BlockType::Todo { .. } => "todo",
            BlockType::Divider { .. } => "divider",
            BlockType::Code { .. } => "code",
            BlockType::Quote { .. } => "quote",
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Block {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    #[serde(flatten)]
    pub r#type: BlockType,
    pub position: u32,
    pub created_at: String,
    pub updated_at: Option<String>,
}
#[derive(Deserialize, Serialize)]
pub struct NewBlock {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    #[serde(flatten)]
    pub r#type: BlockType,
}

#[cfg(test)]
mod test {
    use super::*;

    /// For each of the 11 variants (constructed with dummy payload)
    ///     - serialize the Blocketype
    ///     - assert serde_json::to_value(&bt)["type"] == bt.type_name()
    #[test]
    fn type_name_matches_serde_tag() {
        let bts = vec![
            BlockType::Page {
                title: String::from("foo"),
            },
            BlockType::Text {
                text: String::from("foo"),
            },
            BlockType::Heading1 {
                text: String::from("foo"),
            },
            BlockType::Heading2 {
                text: String::from("foo"),
            },
            BlockType::Heading3 {
                text: String::from("foo"),
            },
            BlockType::BulletedListItem {
                text: String::from("foo"),
            },
            BlockType::NumberedListItem {
                text: String::from("foo"),
            },
            BlockType::Todo {
                text: String::from("foo"),
                checked: false,
            },
            BlockType::Divider {},
            BlockType::Code {
                language: Language::JavaScript,
                content: String::from("foo"),
            },
            BlockType::Quote {
                text: String::from("foo"),
            },
        ];

        for bt in bts {
            assert!(serde_json::to_value(&bt).unwrap()["type"] == bt.type_name())
        }
    }
}
