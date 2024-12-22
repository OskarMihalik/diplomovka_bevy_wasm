//! Run with
//!
//! ```not_rust
//! cargo run -p example-cors
//! ```

use axum::{
    extract::State,
    http::{self, HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use axum_macros::debug_handler;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use dto::test::Test;
use serde::{Deserialize, Serialize};
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
            .route("/newTag", post(create_user))
            .route("/test", post(test_bevy_route))
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

#[debug_handler]
async fn test_bevy_route(Json(payload): Json<Test>) -> (StatusCode, Json<Test>) {
    let mut test = payload.clone();
    test.age += 1;
    (StatusCode::CONFLICT, Json(test))
}

#[debug_handler]
async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    State(pool): State<ConnectionPool>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    username: String,
}
#[derive(Serialize, Deserialize)]
enum FooBar {
    Foo(User),
    Bar(u32),
}
