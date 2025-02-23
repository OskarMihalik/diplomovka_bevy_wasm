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
    routing::{get, post},
    Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use services::{
    model::{get_model_service, get_models_service, upload_new_model_service},
    project::{get_project_service, get_projects_service, insert_project_service},
    tags::{get_tag_service, insert_tag_service, update_tag_service},
    users::{add_user_to_project_service, get_users_in_project_service, get_users_service},
};
use std::net::SocketAddr;
use tokio_postgres::NoTls;
use tower_http::{cors::CorsLayer, services::ServeDir};

type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

#[tokio::main]
async fn main() {
    let manager = PostgresConnectionManager::new_from_stringlike(
        "host=localhost user=postgres password=postgres dbname=bevy port=5438",
        NoTls,
    )
    .unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();
    let backend = async {
        let app = Router::new()
            .nest_service("/assets/model/", ServeDir::new("backend/assets/models"))
            .route("/tags", post(insert_tag_service).patch(update_tag_service))
            .route("/tags/{model_id}", get(get_tag_service))
            .route("/model/{model_id}", get(get_model_service))
            .route("/models/{project_id}", get(get_models_service))
            .route("/project/{project_id}", get(get_project_service))
            .route("/project", get(get_projects_service))
            .route("/project", post(insert_project_service))
            .route("/login", post(services::auth::login))
            .route("/register", post(services::auth::register))
            .route("/users", post(get_users_service))
            .route("/project_user", post(add_user_to_project_service))
            .route("/users/{project_id}", get(get_users_in_project_service))
            .route(
                "/model/{project_id}/{model_name}",
                post(upload_new_model_service).layer(DefaultBodyLimit::max(1024 * 1024 * 1024)),
            )
            .with_state(pool)
            .layer(
                CorsLayer::new()
                    .allow_origin("*".parse::<HeaderValue>().unwrap())
                    .allow_methods([
                        Method::GET,
                        Method::POST,
                        Method::PUT,
                        Method::DELETE,
                        Method::OPTIONS,
                        Method::PATCH,
                    ])
                    .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION]),
            );
        serve(app, 4000).await;
    };

    tokio::join!(backend);
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
