use std::default;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Test {
    pub name: String,
    pub age: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum ErrorReason {
    #[default]
    BadRequest,
    Unauthorized,
    BadCredentials,
    InvalidToken,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorDto {
    pub reason: ErrorReason,
    pub message: String,
}
impl ErrorDto {
    pub fn new(message: String) -> Self {
        ErrorDto {
            reason: ErrorReason::default(),
            message,
        }
    }
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
    pub created_by_id: i32,
    pub email: String,
    pub username: String,
    pub status_title: String,
    pub status_id: i32,
    pub status_color_r: f32,
    pub status_color_g: f32,
    pub status_color_b: f32,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TagMessageDto {
    pub id: i32,
    pub text: String,
    pub created_at: time::PrimitiveDateTime,
    pub updated_at: time::PrimitiveDateTime,
    pub tag_id: i32,
    pub created_by_id: i32,
    pub email: String,
    pub username: String,
    pub is_admin: bool,
}

pub type TagMessagesDtoResponse = Result<Vec<TagMessageDto>, ErrorDto>;
pub type TagMessageDtoResponse = Result<TagMessageDto, ErrorDto>;
pub type CreatedTagMessageDtoResponse = Result<(), ErrorDto>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewTagMessageDto {
    pub text: String,
    pub tag_id: i32,
}
