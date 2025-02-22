use serde::{Deserialize, Serialize};

use crate::default::ErrorDto;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetUsersDto {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OtherUserDto {
    pub id: i32,
    pub email: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectUserDto {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserToProjectDto {
    pub project_id: i32,
    pub user_id: i32,
}

pub type OtherUsersDtoResponse = Result<Vec<OtherUserDto>, ErrorDto>;
pub type ProjectUsersDtoResponse = Result<Vec<ProjectUserDto>, ErrorDto>;
pub type OtherUserDtoResponse = Result<OtherUserDto, ErrorDto>;
