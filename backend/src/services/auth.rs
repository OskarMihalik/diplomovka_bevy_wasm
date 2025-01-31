use crate::{
    auth::{auth_error::ErrorReason, claim::Claims, keys::KEYS},
    ConnectionPool,
};
use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use axum::{extract::State, Json};
use axum_macros::debug_handler;
use chrono::Utc;
use dto::{
    auth::{AuthDtoResponse, LoginDto, RegisterDto, UserDto},
    default::ErrorDto,
};
use jsonwebtoken::{encode, Header};
use model::cornucopia::queries::auth::{insert_user, select_user};
use rand::rngs::OsRng;
use tokio_postgres::GenericClient;

use super::utils::{map_err_pool_con, map_error_reason, map_generic_error, map_ok_to_json};

const PEPPER_SECRET: &str = "mG7JFCuK1/wyoyyaQ1N9lQ";

pub fn create_argon() -> Result<Argon2<'static>, ErrorDto> {
    let a = match Argon2::new_with_secret(
        //pepper.0.expose_secret().as_bytes(),
        PEPPER_SECRET.as_bytes(),
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19000, 2, 1, None).unwrap(),
    ) {
        Ok(ok) => ok,
        Err(err) => {
            return Err(ErrorDto {
                reason: ErrorReason::BadRequest.into(),
                message: err.to_string(),
            })
        }
    };
    return Ok(a);
}

pub fn create_claim(email: String) -> Claims {
    let exp = (Utc::now().naive_utc() + chrono::naive::Days::new(1))
        .and_utc()
        .timestamp() as usize;
    Claims { email, exp }
}

#[debug_handler]
pub async fn login(
    State(pool): State<ConnectionPool>,
    Json(payload): Json<LoginDto>,
) -> Json<AuthDtoResponse> {
    // Check if the user sent the credentials
    // Here, basic verification is used but normally you would use a database

    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let user = match select_user()
        .bind(connection.client(), &payload.email)
        .one()
        .await
    {
        Ok(ok) => ok,
        Err(error) => return map_generic_error(error),
    };

    let argon2 = match create_argon() {
        Ok(ok) => ok,
        Err(err) => {
            return Json(Err(err));
        }
    };
    let password_hash = PasswordHash::new(&user.password).unwrap();
    match argon2.verify_password(payload.password.as_bytes(), &password_hash) {
        Ok(_) => (),
        Err(err) => return map_error_reason(&err.to_string(), ErrorReason::BadCredentials.into()),
    };

    // // create the timestamp for the expiry time - here the expiry time is 1 day
    // // in production you may not want to have such a long JWT life
    let claim = create_claim(payload.email.clone());
    // // Create the authorization token
    let token = match encode(&Header::default(), &claim, &KEYS.encoding) {
        Ok(ok) => ok,
        Err(_) => {
            return Json(AuthDtoResponse::Err(ErrorDto {
                reason: ErrorReason::BadCredentials.into(),
                message: "Bad credentials".to_string(),
            }))
        }
    };

    return map_ok_to_json(UserDto {
        id: user.id,
        email: payload.email,
        username: user.username,
        token,
    });
}

#[debug_handler]
pub async fn register(
    State(pool): State<ConnectionPool>,
    Json(payload): Json<RegisterDto>,
) -> Json<AuthDtoResponse> {
    // Check if the user sent the credentials
    // Here, basic verification is used but normally you would use a database

    // This is the b64 hash of "bad salt!" for demo only: don't do this! Instead use:
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = match create_argon() {
        Ok(ok) => ok,
        Err(err) => {
            return Json(Err(err));
        }
    };
    let hash = match argon2.hash_password(payload.password.as_bytes(), salt.as_salt()) {
        Ok(ok) => ok,
        Err(err) => return map_error_reason(&err.to_string(), ErrorReason::BadRequest.into()),
    };

    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let user_id = match insert_user()
        .bind(
            connection.client(),
            &payload.email,
            &payload.username,
            &hash.serialize().as_str(),
            &salt.as_str(),
        )
        .one()
        .await
    {
        Ok(ok) => ok,
        Err(error) => return map_generic_error(error),
    };

    // // create the timestamp for the expiry time - here the expiry time is 1 day
    // // in production you may not want to have such a long JWT life
    let claim = create_claim(payload.email.clone());

    // // Create the authorization token
    let token = match encode(&Header::default(), &claim, &KEYS.encoding) {
        Ok(ok) => ok,
        Err(err) => {
            return map_error_reason(&err.to_string(), ErrorReason::BadCredentials.into());
        }
    };

    return map_ok_to_json(UserDto {
        id: user_id,
        email: payload.email,
        username: payload.username,
        token,
    });
}
