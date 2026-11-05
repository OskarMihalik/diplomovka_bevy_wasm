//! Run with
//!
//! ```not_rust
//! cargo run -p example-cors
//! ```
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod auth;
mod services;
use axum::{
    extract::DefaultBodyLimit,
    http::{self, HeaderValue, Method},
    routing::{delete, get, patch, post, put},
    Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use dotenv::dotenv;
use services::{
    model::{get_model_service, get_models_service, update_model_service, upload_new_model_service},
    project::{
        delete_project_service, get_project_service, get_projects_service, insert_project_service,
        update_project_service,
    },
    status::{
        create_status_service, delete_status_service, get_statuses_service, update_status_service,
    },
    tag_message::{create_tag_message_service, get_tag_messages_service},
    tags::{delete_tag_service, get_tag_service, insert_tag_service, update_tag_service},
    users::{add_user_to_project_service, get_users_in_project_service, get_users_service},
};
use std::env;
use std::net::SocketAddr;
use tokio_postgres::NoTls;
use tower_http::trace::TraceLayer;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing_subscriber::EnvFilter;
type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

fn main() {
    dotenv().ok(); // Reads the .env file

    // Worker threads come from TOKIO_WORKER_THREADS (read by Tokio). BLOCKING_THREADS caps the
    // spawn_blocking pool (argon2, file reads) like UV_THREADPOOL_SIZE caps Node's libuv pool;
    // Tokio's default of 512 lets every concurrent login compute a 19 MiB argon2 hash at once.
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    if let Ok(value) = env::var("BLOCKING_THREADS") {
        let threads: usize = value
            .parse()
            .expect("BLOCKING_THREADS must be a positive integer");
        builder.max_blocking_threads(threads);
    }
    builder
        .build()
        .expect("failed to build runtime")
        .block_on(run());
}

async fn run() {
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .init();

    let db_port = env::var("DB_POSTGRES_PORT").unwrap();
    let db_postgres_password = env::var("DB_POSTGRES_PASSWORD").unwrap();
    let db_postgres_user = env::var("DB_POSTGRES_USER").unwrap();
    let db_postgres_db = env::var("DB_POSTGRES_DB").unwrap();
    let db_host = env::var("DB_HOST").unwrap();
    let PORT: u16 = env::var("PORT").unwrap().parse::<u16>().unwrap();
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        format!("host={db_host} user={db_postgres_user} password={db_postgres_password} dbname={db_postgres_db} port={db_port}")
            .to_string()
    });

    let manager = PostgresConnectionManager::new_from_stringlike(db_url, NoTls).unwrap();
    // DB_POOL_SIZE defaults to bb8's default of 10 connections
    let pool_size: u32 = env::var("DB_POOL_SIZE").map_or(10, |value| {
        value.parse().expect("DB_POOL_SIZE must be a positive integer")
    });
    let pool = Pool::builder()
        .max_size(pool_size)
        .build(manager)
        .await
        .unwrap();
    println!(
        "backend listening on :{PORT}, tokio workers: {}, blocking threads: {}, db pool: {pool_size}",
        tokio::runtime::Handle::current().metrics().num_workers(),
        env::var("BLOCKING_THREADS").unwrap_or_else(|_| "default".into()),
    );
    let backend = async {
        let app = Router::new()
            .nest_service("/assets/model/", ServeDir::new("backend/assets/models"))
            .nest_service(
                "/assets/attachments/",
                ServeDir::new("backend/assets/attachments"),
            )
            .route("/tags", post(insert_tag_service).patch(update_tag_service))
            .route("/tag_message/{tag_id}", get(get_tag_messages_service))
            .route("/tag_message", post(create_tag_message_service))
            .route("/tags/{model_id}", get(get_tag_service))
            .route("/tag/{tag_id}", delete(delete_tag_service))
            .route("/model/{model_id}", get(get_model_service))
            .route("/model", patch(update_model_service))
            .route("/models/{project_id}", get(get_models_service))
            .route(
                "/project/{project_id}",
                get(get_project_service).delete(delete_project_service),
            )
            .route(
                "/project",
                get(get_projects_service).patch(update_project_service),
            )
            .route("/project", post(insert_project_service))
            .route("/login", post(services::auth::login))
            .route("/register", post(services::auth::register))
            .route("/users", post(get_users_service))
            .route("/project_user", post(add_user_to_project_service))
            .route("/users/{project_id}", get(get_users_in_project_service))
            .route("/statuses/{project_id}", get(get_statuses_service))
            .route("/status", put(create_status_service))
            .route("/status", post(update_status_service))
            .route("/status/{status_id}", delete(delete_status_service))
            .route(
                "/model/{project_id}/{model_name}",
                post(upload_new_model_service).layer(DefaultBodyLimit::max(1024 * 1024 * 1024)),
            )
            .with_state(pool)
            .layer(
                CorsLayer::new()
                    .allow_origin("*".parse::<HeaderValue>().unwrap())
                    .allow_methods(Any)
                    .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION]),
            );
        serve(app, PORT).await;
    };

    tokio::join!(backend);
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
