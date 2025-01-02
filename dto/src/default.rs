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
    pub id: i32,
    pub title: String,
    pub model_id: i64,
    pub created_at: time::OffsetDateTime,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
}

pub type TagDtoResponse = DtoResponse<Vec<TagDto>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewTagDto {
    pub title: String,
    pub model_id: i64,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
}
