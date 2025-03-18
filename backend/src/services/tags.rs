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
use dto::default::{EmptyResponse, ErrorDto, NewTagDto, StatusDto, TagDto, TagDtoResponse};
use model::cornucopia::queries::tags::{
    delete_tag, insert_tag, select_tags, update_tag, DeleteTagStmt, UpdateTagParams,
};
use tokio_postgres::{Client, Error, GenericClient};

use super::utils::{from_shape_to_dto, map_err_pool_con};

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
        Err(error) => return map_err_pool_con(error),
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
        Err(error) => return map_err_pool_con(error),
    };

    let result = insert_tag()
        .bind(
            connection.client(),
            &dto.title,
            &dto.project_id,
            &dto.position_x,
            &dto.position_y,
            &dto.position_z,
            &claims.id,
        )
        .await;

    if let Err(error) = result {
        return Json(TagDtoResponse::Err(ErrorDto::new(format!("{:?}", error))));
    }

    get_tags(connection.client(), &dto.project_id).await
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
                        tag.shape,
                    ) {
                        (
                            Some(title),
                            Some(id),
                            Some(color_r),
                            Some(color_g),
                            Some(color_b),
                            Some(project_id),
                            Some(shape),
                        ) => Some(StatusDto {
                            title,
                            id,
                            color_r,
                            color_g,
                            color_b,
                            project_id,
                            shape: from_shape_to_dto(shape),
                        }),
                        _ => None,
                    };

                    TagDto {
                        id: tag.id,
                        title: tag.title,
                        project_id: tag.project_id,
                        created_at: tag.created_at,
                        position_x: tag.position_x,
                        position_y: tag.position_y,
                        position_z: tag.position_z,
                        created_by_id: tag.created_by_id,
                        email: tag.email,
                        username: tag.username,
                        status_dto,
                        scale_x: tag.scale_x,
                        scale_y: tag.scale_y,
                        scale_z: tag.scale_z,
                        rotation_x: tag.rotation_x,
                        rotation_y: tag.rotation_y,
                        rotation_z: tag.rotation_z,
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
        Err(error) => return map_err_pool_con(error),
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
                scale_x: dto.scale_x,
                scale_y: dto.scale_y,
                scale_z: dto.scale_z,
                rotation_x: dto.rotation_x,
                rotation_y: dto.rotation_y,
                rotation_z: dto.rotation_z,
            },
        )
        .await;

    if let Err(error) = result {
        return Json(TagDtoResponse::Err(ErrorDto::new(format!("{:?}", error))));
    }

    get_tags(connection.client(), &dto.project_id).await
}

#[debug_handler]
pub async fn delete_tag_service(
    State(pool): State<ConnectionPool>,
    Path(tag_id): Path<i32>,
) -> Json<EmptyResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };
    let result = delete_tag().bind(connection.client(), &tag_id).await;

    if let Err(error) = result {
        return Json(Err(ErrorDto::new(format!("{:?}", error))));
    }
    return Json(Ok(()));
}
