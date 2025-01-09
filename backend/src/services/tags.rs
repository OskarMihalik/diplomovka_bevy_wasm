use axum::{
    extract::{Path, State},
    Json,
};
use axum_macros::debug_handler;
// use model::cornucopia::queries::tags::select_tags;

// use crate::ConnectionPool;

use crate::ConnectionPool;
use bb8::RunError;
use dto::default::{ErrorDto, NewTagDto, TagDto, TagDtoResponse};
use model::cornucopia::queries::tags::{insert_tag, select_tags, update_tag};
use tokio_postgres::{Client, Error, GenericClient};

pub fn map_err(error: RunError<Error>) -> Json<TagDtoResponse> {
    Json(TagDtoResponse::Err(ErrorDto {
        message: format!("{:?}", error),
    }))
}

#[debug_handler]
pub async fn get_tag_service(
    Path(model_id): Path<i32>,
    State(pool): State<ConnectionPool>,
) -> Json<TagDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err(error),
    };

    get_tags(connection.client(), &model_id).await
}

#[debug_handler]
pub async fn insert_tag_service(
    State(pool): State<ConnectionPool>,
    Json(dto): Json<NewTagDto>,
) -> Json<TagDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err(error),
    };

    let result = insert_tag()
        .bind(
            connection.client(),
            &dto.title,
            &dto.model_id,
            &dto.position_x,
            &dto.position_y,
            &dto.position_z,
        )
        .await;

    if let Err(error) = result {
        return Json(TagDtoResponse::Err(ErrorDto {
            message: format!("{:?}", error),
        }));
    }

    get_tags(connection.client(), &dto.model_id).await
}

async fn get_tags(client: &Client, model_id: &i32) -> Json<TagDtoResponse> {
    let result = select_tags().bind(client, model_id, &100, &0).all().await;

    match result {
        Ok(tags) => {
            let tag_dtos: Vec<TagDto> = tags
                .into_iter()
                .map(|tag| TagDto {
                    id: tag.id,
                    title: tag.title,
                    model_id: tag.model_id,
                    created_at: tag.created_at,
                    position_x: tag.position_x,
                    position_y: tag.position_y,
                    position_z: tag.position_z,
                })
                .collect();
            Json(TagDtoResponse::Ok(tag_dtos))
        }
        Err(error) => Json(TagDtoResponse::Err(ErrorDto {
            message: format!("{:?}", error),
        })),
    }
}

#[debug_handler]
pub async fn update_tag_service(
    State(pool): State<ConnectionPool>,
    Json(dto): Json<TagDto>,
) -> Json<TagDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err(error),
    };
    let result = update_tag()
        .bind(
            connection.client(),
            &dto.title,
            &dto.position_x,
            &dto.position_y,
            &dto.position_z,
            &dto.id,
        )
        .await;

    if let Err(error) = result {
        return Json(TagDtoResponse::Err(ErrorDto {
            message: format!("{:?}", error),
        }));
    }

    get_tags(connection.client(), &dto.model_id).await
}
