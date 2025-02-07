use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use dto::default::ErrorDto;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum ErrorReason {
    #[default]
    BadRequest,
    Unauthorized,
    BadCredentials,
    InvalidToken,
}

impl From<ErrorReason> for dto::default::ErrorReason {
    fn from(reason: ErrorReason) -> Self {
        match reason {
            ErrorReason::BadRequest => dto::default::ErrorReason::BadRequest,
            ErrorReason::Unauthorized => dto::default::ErrorReason::Unauthorized,
            ErrorReason::BadCredentials => dto::default::ErrorReason::BadCredentials,
            ErrorReason::InvalidToken => dto::default::ErrorReason::InvalidToken,
        }
    }
}

impl IntoResponse for ErrorReason {
    fn into_response(self) -> Response {
        let (reason, error_message) = match self {
            ErrorReason::BadRequest => (ErrorReason::BadRequest, "Bad request"),
            ErrorReason::Unauthorized => {
                (ErrorReason::Unauthorized, "Unauthorized, login to continue")
            }
            ErrorReason::BadCredentials => (ErrorReason::BadCredentials, "Bad credentials"),
            ErrorReason::InvalidToken => (ErrorReason::InvalidToken, "Invalid token"),
        };
        let body: Json<Result<(), ErrorDto>> = Json(Err(ErrorDto {
            reason: reason.into(),
            message: error_message.to_string(),
        }));
        body.into_response()
    }
}
