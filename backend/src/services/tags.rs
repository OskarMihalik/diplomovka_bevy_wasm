use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_macros::debug_handler;
// use model::cornucopia::queries::tags::select_tags;

// use crate::ConnectionPool;

use crate::ConnectionPool;
use bb8::RunError;
use dto::default::{self, ErrorDto, TagDto, TagDtoResponse, Test};
use model::cornucopia::queries::tags::select_tags;
use tokio_postgres::{Error, GenericClient};

pub fn map_err(error: RunError<Error>) -> Json<TagDtoResponse> {
    Json(TagDtoResponse::Err(ErrorDto {
        message: format!("{:?}", error),
    }))
}

#[debug_handler]
pub async fn get_tag_service(State(pool): State<ConnectionPool>) -> Json<TagDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err(error),
    };

    let result = select_tags()
        .bind(connection.client(), &100, &0)
        .all()
        .await;
    match result {
        Ok(tags) => {
            let tag_dtos: Vec<TagDto> = tags
                .into_iter()
                .map(|tag| TagDto {
                    id: tag.id,
                    title: tag.title,
                    model_id: tag.model_id,
                    created_at: tag.created_at,
                })
                .collect();
            return Json(TagDtoResponse::Ok(tag_dtos));
        }
        Err(error) => {
            return Json(TagDtoResponse::Err(ErrorDto {
                message: format!("{:?}", error),
            }))
        }
    }
}
