use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub enum Language {
    JavaScript,
    HTML,
    Python,
    Markdown,
}

#[derive(Serialize)]
#[serde(untagged)]
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
    Divider,
    Code { language: Language, content: String },
}

#[derive(Serialize)]
pub struct Block {
    pub id: Uuid,
    pub r#type: BlockType,
    // pub position: f64,
    // pub created_at: String,
    // pub updated_at: String,
}
