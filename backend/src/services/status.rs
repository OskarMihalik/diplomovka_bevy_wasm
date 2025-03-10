use axum::{
    extract::{Path, State},
    Json,
};
use axum_macros::debug_handler;
use cornucopia_async::Params;
use dto::default::{EmptyResponse, NewStatusDto, StatusDto, StatusesResponse};
use model::cornucopia::queries::status::{
    delete_status, insert_status, select_statuses, update_status, InsertStatusParams,
    SelectStatusesParams, UpdateStatusParams,
};
use tokio_postgres::GenericClient;

use crate::{auth::claim::Claims, ConnectionPool};

use super::utils::{map_err_pool_con, map_generic_error, map_sql_error};

#[debug_handler]
pub async fn get_statuses_service(
    claims: Claims,
    Path(project_id): Path<i32>,
    State(pool): State<ConnectionPool>,
) -> Json<StatusesResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let result = select_statuses()
        .params(
            connection.client(),
            &SelectStatusesParams {
                user_id: claims.id,
                limit: 100,
                offset: 0,
                project_id,
            },
        )
        .all()
        .await;

    let entities = match result {
        Ok(entities) => entities,
        Err(err) => return map_sql_error(err),
    };

    Json(StatusesResponse::Ok(
        entities
            .iter()
            .map(|entity| StatusDto {
                title: entity.title.clone(),
                id: entity.id,
                color_r: entity.color_r,
                color_g: entity.color_g,
                color_b: entity.color_b,
                project_id: entity.project_id,
            })
            .collect(),
    ))
}

#[debug_handler]
pub async fn update_status_service(
    _claims: Claims,
    State(pool): State<ConnectionPool>,
    Json(dto): Json<StatusDto>,
) -> Json<EmptyResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let result = update_status()
        .params(
            connection.client(),
            &UpdateStatusParams {
                title: dto.title.clone(),
                color_r: dto.color_r,
                color_g: dto.color_g,
                color_b: dto.color_b,
                project_id: dto.project_id,
                id: dto.id,
            },
        )
        .await;

    let _count = match result {
        Ok(count) => count,
        Err(err) => return map_sql_error(err),
    };

    if let Err(error) = result {
        return map_generic_error(error);
    }

    return Json(EmptyResponse::Ok(()));
}

#[debug_handler]
pub async fn create_status_service(
    _claims: Claims,
    State(pool): State<ConnectionPool>,
    Json(dto): Json<NewStatusDto>,
) -> Json<EmptyResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let result = insert_status()
        .params(
            connection.client(),
            &InsertStatusParams {
                title: dto.title.clone(),
                color_r: dto.color_r,
                color_g: dto.color_g,
                color_b: dto.color_b,
                project_id: dto.project_id,
            },
        )
        .one()
        .await;

    let _count = match result {
        Ok(count) => count,
        Err(err) => return map_sql_error(err),
    };

    if let Err(error) = result {
        return map_generic_error(error);
    }

    return Json(EmptyResponse::Ok(()));
}

#[debug_handler]
pub async fn delete_status_service(
    _claims: Claims,
    Path(status_id): Path<i32>,
    State(pool): State<ConnectionPool>,
) -> Json<EmptyResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let result = delete_status().bind(connection.client(), &status_id).await;

    let _count = match result {
        Ok(count) => count,
        Err(err) => return map_sql_error(err),
    };

    if let Err(error) = result {
        return map_generic_error(error);
    }

    return Json(EmptyResponse::Ok(()));
}
