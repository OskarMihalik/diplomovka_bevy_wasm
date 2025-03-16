use axum::{
    extract::{Path, State},
    Json,
};
use axum_macros::debug_handler;
// use model::cornucopia::queries::tags::select_tags;
use cornucopia_async::Params;

// use crate::ConnectionPool;

use crate::{auth::claim::Claims, ConnectionPool};
use bb8::RunError;
use dto::default::{ErrorDto, NewTagDto, StatusDto, TagDto, TagDtoResponse};
use model::cornucopia::queries::tags::{insert_tag, select_tags, update_tag, UpdateTagParams};
use tokio_postgres::{Client, Error, GenericClient};

pub fn map_err(error: RunError<Error>) -> Json<TagDtoResponse> {
    Json(TagDtoResponse::Err(ErrorDto::new(format!("{:?}", error))))
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
    claims: Claims,
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
            &claims.id,
        )
        .await;

    if let Err(error) = result {
        return Json(TagDtoResponse::Err(ErrorDto::new(format!("{:?}", error))));
    }

    get_tags(connection.client(), &dto.model_id).await
}

async fn get_tags(client: &Client, model_id: &i32) -> Json<TagDtoResponse> {
    let result = select_tags().bind(client, model_id, &100, &0).all().await;

    match result {
        Ok(tags) => {
            let tag_dtos: Vec<TagDto> = tags
                .into_iter()
                .map(|tag| {
                    let status_dto = match (
                        tag.status_title,
                        tag.status_id,
                        tag.status_color_r,
                        tag.status_color_g,
                        tag.status_color_b,
                        tag.status_project_id,
                    ) {
                        (
                            Some(title),
                            Some(id),
                            Some(color_r),
                            Some(color_g),
                            Some(color_b),
                            Some(project_id),
                        ) => Some(StatusDto {
                            title,
                            id,
                            color_r,
                            color_g,
                            color_b,
                            project_id,
                        }),
                        _ => None,
                    };

                    TagDto {
                        id: tag.id,
                        title: tag.title,
                        model_id: tag.model_id,
                        created_at: tag.created_at,
                        position_x: tag.position_x,
                        position_y: tag.position_y,
                        position_z: tag.position_z,
                        created_by_id: tag.created_by_id,
                        email: tag.email,
                        username: tag.username,
                        status_dto,
                    }
                })
                .collect();
            Json(TagDtoResponse::Ok(tag_dtos))
        }
        Err(error) => Json(TagDtoResponse::Err(ErrorDto::new(format!("{:?}", error)))),
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
        .params(
            connection.client(),
            &UpdateTagParams {
                title: dto.title.clone(),
                position_x: dto.position_x,
                position_y: dto.position_y,
                position_z: dto.position_z,
                id: dto.id,
                status_id: dto.status_dto.map_or(None, |status| Some(status.id)),
            },
        )
        .await;

    if let Err(error) = result {
        return Json(TagDtoResponse::Err(ErrorDto::new(format!("{:?}", error))));
    }

    get_tags(connection.client(), &dto.model_id).await
}
