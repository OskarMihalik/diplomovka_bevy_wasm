use axum::Json;
use bb8::RunError;
use dto::default::ErrorDto;
use tokio_postgres::Error;

pub fn map_err_pool_con<T>(error: RunError<Error>) -> Json<Result<T, ErrorDto>> {
    Json(Err(ErrorDto {
        reason: dto::default::ErrorReason::BadRequest,
        message: format!("{:?}", error),
    }))
}

pub fn map_sql_error<T>(error: Error) -> Json<Result<T, ErrorDto>> {
    return Json(Err(ErrorDto {
        reason: dto::default::ErrorReason::BadRequest,
        message: format!("{:?}", error),
    }));
}

pub fn map_generic_error<T>(error: Error) -> Json<Result<T, ErrorDto>> {
    return Json(Err(ErrorDto {
        reason: dto::default::ErrorReason::BadRequest,
        message: format!("{:?}", error),
    }));
}

pub fn map_error_reason<T>(
    error_str: &str,
    reason: dto::default::ErrorReason,
) -> Json<Result<T, ErrorDto>> {
    return Json(Err(ErrorDto {
        reason,
        message: error_str.to_string(),
    }));
}

pub fn map_ok_to_json<T>(response: T) -> Json<Result<T, ErrorDto>> {
    return Json(Ok(response));
}
