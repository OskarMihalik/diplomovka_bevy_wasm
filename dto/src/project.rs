use serde::{Deserialize, Serialize};

use crate::default::ErrorDto;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectDto {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub created_at: time::PrimitiveDateTime,
    pub updated_at: time::PrimitiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NewProjectDto {
    pub name: String,
    pub description: String,
}

pub type ProjectDtoResponse = Result<ProjectDto, ErrorDto>;
pub type ProjectsDtoResponse = Result<Vec<ProjectDto>, ErrorDto>;
