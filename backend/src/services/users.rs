use axum::{body::Body, extract::State, Json};
use axum_macros::debug_handler;
use dto::{
    default::ErrorDto,
    users::{GetUsersDto, ProjectUserDto, ProjectUserDtoResponse},
};
use model::cornucopia::queries::users::{select_users, select_users_in_project};
use tokio_postgres::GenericClient;

use crate::ConnectionPool;

use super::utils::map_err_pool_con;

#[debug_handler]
pub async fn get_users_service(
    State(pool): State<ConnectionPool>,
    Json(dto): Json<GetUsersDto>,
) -> Json<ProjectUserDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let result = select_users()
        .bind(connection.client(), &dto.email, &100, &0)
        .all()
        .await;
    match result {
        Ok(rows) => {
            let dtos: Vec<ProjectUserDto> = rows
                .into_iter()
                .map(|row| ProjectUserDto {
                    id: row.id,
                    email: row.email,
                    username: row.username,
                })
                .collect();
            return Json(Ok(dtos));
        }
        Err(error) => return Json(Err(ErrorDto::new(format!("{:?}", error)))),
    }
}

// #[debug_handler]
// pub async fn add_user_to_project_service(
//     State(pool): State<ConnectionPool>,
// ) -> Json<ProjectUserDtoResponse> {
// }
