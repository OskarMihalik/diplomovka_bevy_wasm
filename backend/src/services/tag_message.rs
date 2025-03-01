use axum::{
    extract::{Path, State},
    Json,
};
use axum_macros::debug_handler;
use cornucopia_async::Params;
use dto::default::{
    CreatedTagMessageDtoResponse, ErrorDto, NewTagMessageDto, TagMessageDto, TagMessagesDtoResponse,
};
use model::cornucopia::queries::tags::{
    insert_tag_message, select_tag_messages, InsertTagMessageParams, SelectTagMessagesParams,
};
use tokio_postgres::GenericClient;

use crate::{auth::claim::Claims, ConnectionPool};

use super::utils::{map_err_pool_con, map_sql_error};

#[debug_handler]
pub async fn create_tag_message_service(
    claims: Claims,
    State(pool): State<ConnectionPool>,
    Json(dto): Json<NewTagMessageDto>,
) -> Json<CreatedTagMessageDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let affected_rows = match insert_tag_message()
        .params(
            connection.client(),
            &InsertTagMessageParams {
                text: dto.text.clone(),
                tag_id: dto.tag_id,
                created_by_id: claims.id,
            },
        )
        .await
    {
        Ok(ok) => ok,
        Err(error) => return map_sql_error(error),
    };

    if affected_rows == 0 {
        return Json(Err(ErrorDto::new(
            "Failed to insert tag message".to_string(),
        )));
    }

    return Json(Ok(()));
}

#[debug_handler]
pub async fn get_tag_messages_service(
    claims: Claims,
    State(pool): State<ConnectionPool>,
    Path(tag_id): Path<i32>,
) -> Json<TagMessagesDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let tag_messages = match select_tag_messages()
        .params(
            connection.client(),
            &SelectTagMessagesParams {
                tag_id: tag_id,
                user_id: claims.id,
                limit: 0,
                offset: 100,
            },
        )
        .all()
        .await
    {
        Ok(ok) => {
            let dtos = ok
                .iter()
                .map(|tag_message| TagMessageDto {
                    id: tag_message.id,
                    text: tag_message.text.clone(),
                    created_at: tag_message.created_at,
                    updated_at: tag_message.updated_at,
                    created_by_id: tag_message.created_by_id,
                    email: tag_message.email.clone(),
                    username: tag_message.username.clone(),
                    is_admin: tag_message.is_admin,
                    tag_id,
                })
                .collect();
            dtos
        }
        Err(error) => return map_sql_error(error),
    };

    return Json(Ok(tag_messages));
}
