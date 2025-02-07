use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

use jsonwebtoken::{decode, Validation};
use serde::{Deserialize, Serialize};

use super::{auth_error::ErrorReason, keys::KEYS};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub exp: usize,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = ErrorReason;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
            let token = auth_header
                .to_str()
                .map_err(|_| ErrorReason::InvalidToken)?;
            let token_data = decode::<Claims>(token, &KEYS.decoding, &Validation::default())
                .map_err(|_| ErrorReason::InvalidToken)?;
            Ok(token_data.claims)
        } else {
            Err(ErrorReason::Unauthorized)
        }
    }
}
