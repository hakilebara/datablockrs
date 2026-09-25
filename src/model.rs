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
pub enum BlockType {
    Page { title: String },
    Text,
    Heading1,
    Heading2,
    Heading3,
    BulletedListItem,
    NumberedListItem,
    Todo { checked: bool },
    Quote,
    Divider,
    Code { language: Language },
}

#[derive(Serialize)]
pub struct Block {
    pub id: Uuid,
    pub block_type: BlockType,
}
