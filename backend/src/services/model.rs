use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_macros::debug_handler;
// use model::cornucopia::queries::tags::select_tags;

// use crate::ConnectionPool;

use crate::ConnectionPool;
use bb8::RunError;
use dto::{
    default::{self, ErrorDto, NewTagDto, TagDto, TagDtoResponse, Test},
    model::{ModelDto, ModelDtoResponse},
};
use model::cornucopia::queries::tags::{insert_tag, select_model, select_tags};
use tokio_postgres::{Error, GenericClient};

pub fn map_err(error: RunError<Error>) -> Json<ModelDtoResponse> {
    Json(ModelDtoResponse::Err(ErrorDto {
        message: format!("{:?}", error),
    }))
}

#[debug_handler]
pub async fn get_model_service(
    Path(model_id): Path<i32>,
    State(pool): State<ConnectionPool>,
) -> Json<ModelDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err(error),
    };

    let result = select_model()
        .bind(connection.client(), &model_id, &100, &0)
        .one()
        .await;
    match result {
        Ok(model) => {
            let dto = ModelDto {
                id: model.id,
                version: model.version,
                model_link: model.model_link,
                name: model.name,
                created_at: model.created_at,
                updated_at: model.updated_at,
                project_id: model.project_id,
            };
            return Json(ModelDtoResponse::Ok(dto));
        }
        Err(error) => {
            return Json(ModelDtoResponse::Err(ErrorDto {
                message: format!("{:?}", error),
            }))
        }
    }
}
