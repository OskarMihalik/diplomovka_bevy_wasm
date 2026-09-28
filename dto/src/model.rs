use serde::{Deserialize, Serialize};

use crate::default::ErrorDto;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelDto {
    pub id: i32,
    pub version: i32,
    pub model_link: String,
    pub name: String,
    pub created_at: time::PrimitiveDateTime,
    pub updated_at: time::PrimitiveDateTime,
    pub project_id: i32,
}

pub type ModelDtoResponse = Result<ModelDto, ErrorDto>;

pub type ModelsDtoResponse = Result<Vec<ModelDto>, ErrorDto>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateModelDto {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewModelDto {
    pub name: String,
    pub project_id: i32,
}

impl Default for NewModelDto {
    fn default() -> Self {
        NewModelDto {
            name: String::new(),
            project_id: 0,
        }
    }
}
