//! Run with
//!
//! ```not_rust
//! cargo run -p example-cors
//! ```

mod services;
use axum::{
    extract::State,
    http::{self, HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use services::{
    model::get_model_service,
    tags::{get_tag_service, insert_tag_service},
};
use std::net::SocketAddr;
use tokio_postgres::NoTls;
use tower_http::cors::CorsLayer;

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
            .route("/tags", get(get_tag_service).post(insert_tag_service))
            .route("/model/:model_id", get(get_model_service))
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
                    ])
                    .allow_headers([http::header::CONTENT_TYPE]),
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
