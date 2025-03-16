use axum::Json;
use bb8::RunError;
use dto::default::{ErrorDto, Shape};
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

pub fn from_shape_to_dto(shape: model::cornucopia::types::public::Shape) -> Shape {
    match shape {
        model::cornucopia::types::public::Shape::Cuboid => Shape::Cuboid,
        model::cornucopia::types::public::Shape::Tetrahedron => Shape::Tetrahedron,
        model::cornucopia::types::public::Shape::Capsule3d => Shape::Capsule3d,
        model::cornucopia::types::public::Shape::Torus => Shape::Torus,
        model::cornucopia::types::public::Shape::Cylinder => Shape::Cylinder,
        model::cornucopia::types::public::Shape::Cone => Shape::Cone,
        model::cornucopia::types::public::Shape::ConicalFrustum => Shape::ConicalFrustum,
        model::cornucopia::types::public::Shape::Sphere => Shape::Sphere,
    }
}

pub fn from_dto_to_shape(shape: Shape) -> model::cornucopia::types::public::Shape {
    match shape {
        Shape::Cuboid => model::cornucopia::types::public::Shape::Cuboid,
        Shape::Tetrahedron => model::cornucopia::types::public::Shape::Tetrahedron,
        Shape::Capsule3d => model::cornucopia::types::public::Shape::Capsule3d,
        Shape::Torus => model::cornucopia::types::public::Shape::Torus,
        Shape::Cylinder => model::cornucopia::types::public::Shape::Cylinder,
        Shape::Cone => model::cornucopia::types::public::Shape::Cone,
        Shape::ConicalFrustum => model::cornucopia::types::public::Shape::ConicalFrustum,
        Shape::Sphere => model::cornucopia::types::public::Shape::Sphere,
    }
}
