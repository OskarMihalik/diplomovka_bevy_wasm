use serde::{Deserialize, Serialize};

use crate::default::ErrorDto;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDto {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub token: String,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RegisterDto {
    pub email: String,
    pub username: String,
    pub password: String,
}

impl RegisterDto {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

impl LoginDto {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

pub type AuthDtoResponse = Result<UserDto, ErrorDto>;
