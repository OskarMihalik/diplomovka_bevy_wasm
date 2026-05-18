//! Run with
//!
//! ```not_rust
//! cargo run -p example-cors
//! ```

mod auth;
mod services;
use axum::{
    extract::DefaultBodyLimit,
    http::{self, HeaderValue, Method},
    routing::{delete, get, post, put},
    Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use dotenv::dotenv;
use services::{
    model::{get_model_service, get_models_service, upload_new_model_service},
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
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

#[tokio::main]
async fn main() {
    dotenv().ok(); // Reads the .env file
    let db_port = env::var("DB_PORT").unwrap();
    let db_postgres_password = env::var("DB_POSTGRES_PASSWORD").unwrap();
    let db_postgres_user = env::var("DB_POSTGRES_USER").unwrap();
    let db_postgres_db = env::var("DB_POSTGRES_DB").unwrap();
    let db_host = env::var("DB_HOST").unwrap();
    let backend_port: u16 = env::var("BACKEND_PORT").unwrap().parse::<u16>().unwrap();
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        format!("host={db_host} user={db_postgres_user} password={db_postgres_password} dbname={db_postgres_db} port={db_port}")
            .to_string()
    });

    let manager = PostgresConnectionManager::new_from_stringlike(db_url, NoTls).unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();
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
        serve(app, backend_port).await;
    };

    tokio::join!(backend);
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
