use serde::{Deserialize, Serialize};

use crate::default::ErrorDto;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetUsersDto {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectUserDto {
    pub id: i32,
    pub email: String,
    pub username: String,
}

pub type ProjectUserDtoResponse = Result<Vec<ProjectUserDto>, ErrorDto>;
