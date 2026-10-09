//! Minimal Axum server for experiment A (framework without a database).
//! Every endpoint mirrors `framework_bench/express/src/index.ts`; see `framework_bench/PLAN.md`.

use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version, password_hash::SaltString};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Query},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tower_http::services::ServeFile;

// Same pepper and parameters as backend/src/services/auth.rs and backend-express.
const PEPPER_SECRET: &str = "mG7JFCuK1/wyoyyaQ1N9lQ";
// Fixed salt, so both servers must return the byte-identical PHC string.
const SALT: &[u8; 16] = b"framework-bench!";
const PASSWORD: &str = "user@mail.com";

const LARGE_JSON_ITEMS: u32 = 1600;
const DESCRIPTION: &str = "Window frame on the north facade does not match the detail drawing; check the lintel height and the sill offset before ordering.";
const STATUSES: [&str; 3] = ["TODO", "IN_PROGRESS", "DONE"];
const MAX_DELAY_MS: u64 = 10_000;
const BODY_LIMIT: usize = 16 * 1024 * 1024;

#[derive(Serialize)]
struct Message {
    message: &'static str,
}

#[derive(Serialize)]
struct Position {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Serialize)]
struct Tag {
    id: u32,
    project_id: u32,
    title: String,
    description: &'static str,
    status: &'static str,
    created_by: &'static str,
    position: Position,
}

#[derive(Deserialize, Serialize)]
struct EchoItem {
    name: String,
    value: f64,
}

#[derive(Deserialize)]
struct EchoIn {
    id: i64,
    title: String,
    items: Vec<EchoItem>,
}

#[derive(Serialize)]
struct EchoOut {
    id: i64,
    title: String,
    count: usize,
    sum: f64,
    items: Vec<EchoItem>,
}

#[derive(Deserialize)]
struct DelayQuery {
    ms: Option<u64>,
}

#[derive(Serialize)]
struct UploadOut {
    bytes: usize,
}

fn large_json_items() -> Vec<Tag> {
    (0..LARGE_JSON_ITEMS)
        .map(|i| {
            let f = i as f64;
            Tag {
                id: i,
                project_id: i / 50 + 1,
                title: format!("Tag {i}"),
                description: DESCRIPTION,
                status: STATUSES[(i % 3) as usize],
                created_by: "user@mail.com",
                position: Position {
                    x: f * 0.25,
                    y: f * 0.5 - 100.0,
                    z: f * 0.125,
                },
            }
        })
        .collect()
}

async fn plaintext() -> &'static str {
    "Hello, World!"
}

async fn json() -> Json<Message> {
    Json(Message {
        message: "Hello, World!",
    })
}

async fn echo_json(Json(input): Json<EchoIn>) -> Json<EchoOut> {
    let sum = input.items.iter().map(|item| item.value).sum();
    let mut items = input.items;
    items.reverse();
    Json(EchoOut {
        id: input.id,
        title: input.title.to_uppercase(),
        count: items.len(),
        sum,
        items,
    })
}

// Serialized on every request from shared data, without cloning it.
async fn json_large(items: Arc<Vec<Tag>>) -> impl IntoResponse {
    match serde_json::to_vec(&*items) {
        Ok(body) => ([(header::CONTENT_TYPE, "application/json")], body).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn cpu() -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(|| {
        let argon2 = Argon2::new_with_secret(
            PEPPER_SECRET.as_bytes(),
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(19000, 2, 1, Some(32)).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let salt = SaltString::encode_b64(SALT).map_err(|e| e.to_string())?;
        argon2
            .hash_password(PASSWORD.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| e.to_string())
    })
    .await;
    match result {
        Ok(Ok(hash)) => hash.into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn delay(Query(query): Query<DelayQuery>) -> &'static str {
    let ms = query.ms.unwrap_or(20).min(MAX_DELAY_MS);
    tokio::time::sleep(Duration::from_millis(ms)).await;
    "ok"
}

async fn upload(body: Bytes) -> Json<UploadOut> {
    Json(UploadOut { bytes: body.len() })
}

async fn shutdown_signal() {
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("failed to install SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = sigterm.recv() => {},
    }
}

fn main() {
    // Worker threads come from TOKIO_WORKER_THREADS (read by Tokio). BLOCKING_THREADS caps the
    // spawn_blocking pool (argon2, file reads) like UV_THREADPOOL_SIZE caps Node's libuv pool;
    // with Tokio's default of 512, 256 concurrent argon2 hashes (19 MiB each) exceed the memory limit.
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    if let Ok(value) = std::env::var("BLOCKING_THREADS") {
        let threads: usize = value.parse().expect("BLOCKING_THREADS must be a positive integer");
        builder.max_blocking_threads(threads);
    }
    builder.build().expect("failed to build runtime").block_on(serve());
}

async fn serve() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let file_path = std::env::var("FILE_PATH")
        .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../../loadtest/src/loadtest.glb").to_string());
    if !std::path::Path::new(&file_path).is_file() {
        panic!("FILE_PATH {file_path} does not exist");
    }

    let items = Arc::new(large_json_items());

    let app = Router::new()
        .route("/plaintext", get(plaintext))
        .route("/json", get(json))
        .route("/echo-json", post(echo_json))
        .route("/json-large", get(move || json_large(items.clone())))
        .route("/cpu", get(cpu))
        .route("/delay", get(delay))
        .route_service("/file", ServeFile::new(&file_path))
        .route("/upload", post(upload))
        .layer(DefaultBodyLimit::max(BODY_LIMIT));

    // TCP_NODELAY stays off (axum::serve default, same as backend/); Express disables it to match.
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("failed to bind");

    let workers = tokio::runtime::Handle::current().metrics().num_workers();
    let blocking = std::env::var("BLOCKING_THREADS").unwrap_or_else(|_| "default".into());
    println!("axum listening on :{port}, tokio workers: {workers}, blocking threads: {blocking}, file: {file_path}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}
