use serde::{Deserialize, Serialize};

use crate::default::ErrorDto;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDto {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub token: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterDto {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

pub type RegisterDtoResponse = Result<UserDto, ErrorDto>;
