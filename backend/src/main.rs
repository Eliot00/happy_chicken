use axum::{extract::Extension, http::StatusCode, routing::get, Json, Router};
use entity::food;
use sea_orm::{prelude::*, Database, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "backend=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL").expect("Can't read DATABASE_URL");
    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");

    let app = Router::new()
        .route("/foods", get(get_all_foods).post(create_food))
        .layer(Extension(conn));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_all_foods(
    Extension(ref conn): Extension<DatabaseConnection>,
) -> Result<Json<Vec<food::Model>>, (StatusCode, String)> {
    food::Entity::find()
        .order_by_asc(food::Column::Id)
        .all(conn)
        .await
        .map(|value| Json(value))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct CreateFoodRequest {
    pub name: String,
    pub price: f32,
}

async fn create_food(
    Extension(ref conn): Extension<DatabaseConnection>,
    Json(req): Json<CreateFoodRequest>,
) -> Result<Json<food::Model>, (StatusCode, String)> {
    let model = food::ActiveModel {
        name: Set(req.name.clone()),
        price: Set(req.price),
        ..Default::default()
    };
    model
        .insert(conn)
        .await
        .map(|value| Json(value))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
