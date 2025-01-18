use axum::{
    body::Bytes,
    extract::{Multipart, Path, State},
    http::StatusCode,
    BoxError, Json,
};
use axum_macros::debug_handler;
use futures::{Stream, TryStreamExt};
use std::io;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;

use crate::ConnectionPool;
use dto::{
    default::ErrorDto,
    model::{ModelDto, ModelDtoResponse, ModelsDtoResponse},
};
use model::cornucopia::queries::tags::{insert_model, select_model, select_models};
use tokio_postgres::GenericClient;

use super::utils::map_err_pool_con;

pub const UPLOADS_DIRECTORY: &str = "backend/assets/models";

#[debug_handler]
pub async fn get_model_service(
    Path(model_id): Path<i32>,
    State(pool): State<ConnectionPool>,
) -> Json<ModelDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let result = select_model()
        .bind(connection.client(), &model_id, &100, &0)
        .one()
        .await;
    match result {
        Ok(model) => {
            let dto = ModelDto {
                id: model.id,
                version: model.version,
                model_link: model.model_link,
                name: model.name,
                created_at: model.created_at,
                updated_at: model.updated_at,
                project_id: model.project_id,
            };
            return Json(ModelDtoResponse::Ok(dto));
        }
        Err(error) => {
            return Json(ModelDtoResponse::Err(ErrorDto {
                message: format!("{:?}", error),
            }))
        }
    }
}

#[debug_handler]
pub async fn get_models_service(
    Path(project_id): Path<i32>,
    State(pool): State<ConnectionPool>,
) -> Json<ModelsDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let result = select_models()
        .bind(connection.client(), &project_id, &100, &0)
        .all()
        .await;

    match result {
        Ok(models) => {
            let dtos: Vec<ModelDto> = models
                .iter()
                .map(|model| ModelDto {
                    id: model.id,
                    version: model.version,
                    model_link: model.model_link.clone(),
                    name: model.name.clone(),
                    created_at: model.created_at,
                    updated_at: model.updated_at,
                    project_id: model.project_id,
                })
                .collect();
            return Json(ModelsDtoResponse::Ok(dtos));
        }
        Err(error) => {
            return Json(ModelsDtoResponse::Err(ErrorDto {
                message: format!("{:?}", error),
            }))
        }
    }
}

// TODO: what if success to db but not succesfull to storage
#[debug_handler]
pub async fn upload_new_model_service(
    Path((project_id, model_name)): Path<(i32, String)>,
    State(pool): State<ConnectionPool>,
    mut multipart: Multipart,
) -> Json<ModelsDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => {
            return Json(ModelsDtoResponse::Err(ErrorDto {
                message: format!("{:?}", error),
            }))
        }
    };

    let inserted_model_id = match insert_model()
        .bind(
            connection.client(),
            &1,
            &"model_link",
            &model_name,
            &project_id,
        )
        .one()
        .await
    {
        Ok(ok) => ok,
        Err(error) => {
            return Json(ModelsDtoResponse::Err(ErrorDto {
                message: format!("{:?}", error),
            }))
        }
    };

    while let Ok(Some(field)) = multipart.next_field().await {
        let _file_name = if let Some(file_name) = field.file_name() {
            file_name.to_owned()
        } else {
            continue;
        };

        match stream_to_file(&format!("{inserted_model_id}.glb"), field).await {
            Ok(ok) => ok,
            Err(err) => {
                return Json(Err(ErrorDto {
                    message: format!("{:?}", err),
                }))
            }
        };
    }

    let result = select_models()
        .bind(connection.client(), &project_id, &100, &0)
        .all()
        .await;

    match result {
        Ok(list) => {
            let dtos: Vec<ModelDto> = list
                .iter()
                .map(|model| ModelDto {
                    id: model.id,
                    version: model.version,
                    model_link: model.model_link.clone(),
                    name: model.name.clone(),
                    created_at: model.created_at,
                    updated_at: model.updated_at,
                    project_id: model.project_id,
                })
                .collect();
            return Json(ModelsDtoResponse::Ok(dtos));
        }
        Err(error) => {
            return Json(ModelsDtoResponse::Err(ErrorDto {
                message: format!("{:?}", error),
            }))
        }
    }
}

// Save a `Stream` to a file
async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(path) {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    }

    async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        let path = std::path::Path::new(UPLOADS_DIRECTORY).join(path);
        let mut file = BufWriter::new(File::create(path).await?);
        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

// to prevent directory traversal attacks we ensure the path consists of exactly one normal
// component
fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}
