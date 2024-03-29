use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use tracing::debug;

use crate::{model::ModelManager, web::Result};

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/api/subscribe", post(api_subscribe))
        .with_state(mm)
        .route("/health-check", get(health_check))
}

async fn health_check() -> StatusCode {
    debug!("{:<20} - GET /health-check", "HANDLER");
    StatusCode::OK
}

#[derive(Deserialize, Debug)]
struct Subscriber {
    pub name: String,
    pub email: String,
}

async fn api_subscribe(
    State(mm): State<ModelManager>,
    Json(subscriber): Json<Subscriber>,
) -> Result<StatusCode> {
    debug!("{:<20} - POST /api/subscribe", "HANDLER");
    let db = mm.db();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (email, name, subscribed_at)
        VALUES ($1, $2, $3)
    "#,
        subscriber.email,
        subscriber.name,
        Utc::now()
    )
    .execute(db)
    .await?;

    Ok(StatusCode::OK)
}
