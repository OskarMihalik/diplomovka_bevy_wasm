use axum::Json;
use axum_macros::debug_handler;
use chrono::Utc;
use dto::{
    auth::{LoginDto, RegisterDto, RegisterDtoResponse, UserDto},
    default::ErrorDto,
};
use jsonwebtoken::{encode, Header};

use crate::auth::{auth_error::ErrorReason, claim::Claims, keys::KEYS};

#[debug_handler]
pub async fn login(Json(payload): Json<LoginDto>) -> Json<RegisterDtoResponse> {
    // Check if the user sent the credentials
    // Here, basic verification is used but normally you would use a database
    if &payload.email != "foo" || &payload.password != "bar" {
        return Json(Err(ErrorDto {
            reason: ErrorReason::BadCredentials.into(),
            message: "Bad credentials".to_string(),
        }));
    }

    // // create the timestamp for the expiry time - here the expiry time is 1 day
    // // in production you may not want to have such a long JWT life
    // chrono::naive::Days::new(1)).timestamp()
    // let next_day= time::Date::next_day().unwrap().with_time(time)
    let exp = (Utc::now().naive_utc() + chrono::naive::Days::new(1)).timestamp() as usize;
    let claims = Claims {
        email: payload.email.clone(),
        exp,
    };
    // // Create the authorization token
    let token = match encode(&Header::default(), &claims, &KEYS.encoding) {
        Ok(ok) => ok,
        Err(_) => {
            return Json(RegisterDtoResponse::Err(ErrorDto {
                reason: ErrorReason::BadCredentials.into(),
                message: "Bad credentials".to_string(),
            }))
        }
    };

    // Send the authorized token
    Json(RegisterDtoResponse::Ok(UserDto {
        id: 0,
        email: payload.email,
        username: "sadfasdf".to_string(),
        token,
    }))
}
