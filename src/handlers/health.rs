use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use sqlx::PgPool;

pub async fn root_handler() -> &'static str {
    "Server running!"
}

pub async fn health_handler(State(pool): State<PgPool>) -> impl IntoResponse {
    let db_status = match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    Json(json!({
        "status": if db_status == "connected" { "UP" } else { "DEGRADED" },
        "database": db_status,
        "version": "1.0.0-rust"
    }))
}

pub async fn health_ready(State(pool): State<PgPool>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => Json(json!({"ready": true})),
        Err(_) => Json(json!({"ready": false})),
    }
}

pub async fn health_live() -> impl IntoResponse {
    Json(json!({"live": true}))
}

pub async fn system_info() -> impl IntoResponse {
    Json(json!({
        "service": "inventory-backend",
        "runtime": "Rust / Tokio / Axum",
        "database": "PostgreSQL 17",
        "architecture": "Clean Architecture / Event-Ledger"
    }))
}
