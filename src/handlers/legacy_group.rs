use crate::{
    auth::AuthenticatedUser,
    errors::AppError,
    handlers::{
        categories, dashboard, inventory,
        products::{self, ProductQuery},
    },
    models::{category::CategoryRequest, movement::CreateMovementRequest, product::ProductRequest},
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::PgPool;
use std::fs;
use uuid::Uuid;

pub async fn list_products(
    State(pool): State<PgPool>,
    auth: AuthenticatedUser,
    Path(_group_id): Path<String>,
    Query(query): Query<ProductQuery>,
) -> Result<impl IntoResponse, AppError> {
    products::list_products(State(pool), auth, Query(query)).await
}

pub async fn create_product(
    State(pool): State<PgPool>,
    auth: AuthenticatedUser,
    Path(_group_id): Path<String>,
    Json(payload): Json<ProductRequest>,
) -> Result<impl IntoResponse, AppError> {
    products::create_product(State(pool), auth, Json(payload)).await
}

pub async fn get_product(
    State(pool): State<PgPool>,
    auth: AuthenticatedUser,
    Path((_group_id, id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    products::get_product(State(pool), auth, Path(id)).await
}

pub async fn update_product(
    State(pool): State<PgPool>,
    auth: AuthenticatedUser,
    Path((_group_id, id)): Path<(String, String)>,
    Json(payload): Json<ProductRequest>,
) -> Result<impl IntoResponse, AppError> {
    products::update_product(State(pool), auth, Path(id), Json(payload)).await
}

pub async fn delete_product(
    State(pool): State<PgPool>,
    auth: AuthenticatedUser,
    Path((_group_id, id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    products::delete_product(State(pool), auth, Path(id)).await
}

pub async fn upload_image(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Path((_group_id, product_id)): Path<(String, String)>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut file_url = None;
    fs::create_dir_all("uploads").map_err(|e| AppError::Internal(e.to_string()))?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "image" || name == "file" {
            let filename = format!("{}_{}.jpg", product_id, Uuid::new_v4());
            let filepath = format!("uploads/{}", filename);
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
            fs::write(&filepath, data).map_err(|e| AppError::Internal(e.to_string()))?;
            file_url = Some(format!("/uploads/{}", filename));
            break;
        }
    }

    if let Some(ref url) = file_url {
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query("UPDATE product SET image_uri = $1, last_updated = $2, server_updated_at = $2 WHERE product_id = $3")
            .bind(url)
            .bind(now)
            .bind(&product_id)
            .execute(&pool)
            .await?;

        Ok((StatusCode::OK, url.clone()))
    } else {
        Err(AppError::BadRequest(
            "No image data provided in multipart upload".to_string(),
        ))
    }
}

pub async fn list_categories(
    State(pool): State<PgPool>,
    auth: AuthenticatedUser,
    Path(_group_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    categories::list_categories(State(pool), auth).await
}

pub async fn create_category(
    State(pool): State<PgPool>,
    auth: AuthenticatedUser,
    Path(_group_id): Path<String>,
    Json(payload): Json<CategoryRequest>,
) -> Result<impl IntoResponse, AppError> {
    categories::create_category(State(pool), auth, Json(payload)).await
}

pub async fn list_movements(
    State(pool): State<PgPool>,
    auth: AuthenticatedUser,
    Path(_group_id): Path<String>,
    Query(query): Query<inventory::MovementQuery>,
) -> Result<impl IntoResponse, AppError> {
    inventory::list_movements(State(pool), auth, Query(query)).await
}

pub async fn create_movement(
    State(pool): State<PgPool>,
    auth: AuthenticatedUser,
    Path(_group_id): Path<String>,
    Json(payload): Json<CreateMovementRequest>,
) -> Result<impl IntoResponse, AppError> {
    inventory::create_movement_generic(State(pool), auth, Json(payload)).await
}

pub async fn get_summary(
    State(pool): State<PgPool>,
    auth: AuthenticatedUser,
    Path(_group_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    dashboard::get_dashboard_summary(State(pool), auth).await
}
