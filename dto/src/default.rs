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
pub struct TagDto {
    pub id: i32,
    pub title: String,
    pub model_id: i32,
    pub created_at: time::PrimitiveDateTime,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
}

pub type TagDtoResponse = Result<Vec<TagDto>, ErrorDto>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewTagDto {
    pub title: String,
    pub model_id: i32,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
}
