use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;

mod follows;
mod tweets;
mod users;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

let app = Router::new()
    .route("/users", get(get_users).post(create_user))
    .route("/users/{username}", get(get_user))
    .route("/users/{username}/tweets", get(get_user_tweets))
    .route("/tweets", get(get_tweets).post(create_tweet))
    .route("/tweets/{id}", axum::routing::delete(delete_tweet))
    .route("/tweets/feed/{user_id}", get(get_feed))
    .route("/follows/toggle", post(toggle_follow))
    .layer(CorsLayer::permissive())
    .with_state(pool);

    let addr = "0.0.0.0:3000";
    println!("Server running at http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler functions for the routes user

async fn get_users(State(pool): State<PgPool>) -> Result<Json<Vec<users::User>>, StatusCode> {
    users::get_all(&pool)
    .await
    .map(Json)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_user(
    State(pool): State<PgPool>,
    Path(username): Path<String>,
) -> Result<Json<users::User>, StatusCode> {
    users::get_by_username(&pool, &username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create_user(
    State(pool): State<PgPool>,
    Json(input): Json<users::CreateUser>,
)-> Result<(StatusCode, Json<users::User>), StatusCode> {
    users::create(&pool, input)
        .await
        .map(|user| (StatusCode::CREATED, Json(user)))
        .map_err(|_| StatusCode::BAD_REQUEST)
}

// Handler functions for the routes tweets

async fn get_tweets(
    State(pool): State<PgPool>
) -> Result<Json<Vec<tweets::Tweet>>, StatusCode> {
    tweets::get_all(&pool)
    .await
    .map(Json)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_user_tweets(
    State(pool): State<PgPool>,
    Path(username): Path<String>,
) -> Result<Json<Vec<tweets::Tweet>>, StatusCode> {
    tweets::get_by_username(&pool, &username)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_feed(
    State(pool): State<PgPool>,
    Path(user_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<tweets::Tweet>>, StatusCode> {
    tweets::get_feed(&pool, user_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_tweet(
    State(pool): State<PgPool>,
    Json(input): Json<tweets::CreateTweet>,
) -> Result<(StatusCode, Json<tweets::Tweet>), StatusCode> {
    tweets::create(&pool, input)
        .await
        .map(|tweet| (StatusCode::CREATED, Json(tweet)))
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn delete_tweet(
    State(pool): State<PgPool>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, StatusCode> {
    tweets::delete(&pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|_| StatusCode::NO_CONTENT)
        .ok_or(StatusCode::NOT_FOUND)
}

// Handler functions for the routes follows

async fn toggle_follow(
    State(pool): State<PgPool>,
    Json(input): Json<follows::ToggleFollow>,
) -> Result<Json<follows::FollowResult>, StatusCode> {
    follows::toggle(&pool, input)
        .await
        .map(|is_following| Json(follows::FollowResult { is_following }))
        .map_err(|_| StatusCode::BAD_REQUEST)
}