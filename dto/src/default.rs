use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Test {
    pub name: String,
    pub age: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorDto {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DtoResponse<T> {
    Ok(T),
    Err(ErrorDto),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TagDto {
    pub id: i64,
    pub title: String,
    pub model_id: i64,
    pub created_at: time::OffsetDateTime,
}

pub type TagDtoResponse = DtoResponse<Vec<TagDto>>;
