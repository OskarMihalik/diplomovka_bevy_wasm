use axum::Json;
use bb8::RunError;
use dto::default::ErrorDto;
use tokio_postgres::Error;

pub fn map_err_pool_con<T>(error: RunError<Error>) -> Json<Result<T, ErrorDto>> {
    Json(Err(ErrorDto {
        message: format!("{:?}", error),
    }))
}

pub fn map_sql_error<T>(error: Error) -> Json<Result<T, ErrorDto>> {
    return Json(Err(ErrorDto {
        message: format!("{:?}", error),
    }));
}
