use crate::{
    auth::AuthenticatedUser,
    errors::AppError,
    models::category::{Category, CategoryRequest, CategoryResponse},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn list_categories(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT category_id, group_id, name, description, created_at, last_updated, server_updated_at FROM category ORDER BY name ASC"
    )
    .fetch_all(&pool)
    .await?;

    let responses: Vec<CategoryResponse> =
        categories.into_iter().map(CategoryResponse::from).collect();
    Ok((StatusCode::OK, Json(responses)))
}

pub async fn get_category(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let cat = sqlx::query_as::<_, Category>(
        "SELECT category_id, group_id, name, description, created_at, last_updated, server_updated_at FROM category WHERE category_id = $1",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Category '{}' not found", id)))?;

    Ok((StatusCode::OK, Json(CategoryResponse::from(cat))))
}

pub async fn create_category(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Json(payload): Json<CategoryRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation(
            "Category name cannot be empty".to_string(),
        ));
    }

    let now = chrono::Utc::now().timestamp_millis();
    let category_id = payload
        .category_id
        .unwrap_or_else(|| format!("category-{}", Uuid::new_v4()));
    let default_group = "default-grocery-group";

    sqlx::query(
        r#"
        INSERT INTO category (category_id, group_id, name, description, created_at, last_updated, server_updated_at)
        VALUES ($1, $2, $3, $4, $5, $5, $5)
        ON CONFLICT (category_id) DO UPDATE SET name = $3, description = $4, last_updated = $5, server_updated_at = $5
        "#
    )
    .bind(&category_id)
    .bind(default_group)
    .bind(payload.name.trim())
    .bind(payload.description.as_deref().map(|d| d.trim()))
    .bind(now)
    .execute(&pool)
    .await?;

    let response = CategoryResponse {
        category_id,
        group_id: default_group.to_string(),
        name: payload.name.trim().to_string(),
        description: payload.description,
        created_at: now,
        last_updated: now,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_category(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<CategoryRequest>,
) -> Result<impl IntoResponse, AppError> {
    let now = chrono::Utc::now().timestamp_millis();

    let res = sqlx::query(
        r#"
        UPDATE category
        SET name = $1, description = $2, last_updated = $3, server_updated_at = $3
        WHERE category_id = $4
        "#,
    )
    .bind(payload.name.trim())
    .bind(payload.description.as_deref().map(|d| d.trim()))
    .bind(now)
    .bind(&id)
    .execute(&pool)
    .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Category '{}' not found", id)));
    }

    let cat = sqlx::query_as::<_, Category>(
        "SELECT category_id, group_id, name, description, created_at, last_updated, server_updated_at FROM category WHERE category_id = $1"
    )
    .bind(&id)
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::OK, Json(CategoryResponse::from(cat))))
}

pub async fn delete_category(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let now = chrono::Utc::now().timestamp_millis();
    let row = sqlx::query("SELECT group_id FROM category WHERE category_id = $1")
        .bind(&id)
        .fetch_optional(&pool)
        .await?;

    if let Some(r) = row {
        let group_id: String = r.get("group_id");

        sqlx::query(
            "INSERT INTO deleted (entity_id, group_id, entity_type, deleted_at) VALUES ($1, $2, 'CATEGORY', $3) ON CONFLICT (entity_id, entity_type) DO UPDATE SET deleted_at = $3"
        )
        .bind(&id)
        .bind(&group_id)
        .bind(now)
        .execute(&pool)
        .await?;

        sqlx::query("DELETE FROM category WHERE category_id = $1")
            .bind(&id)
            .execute(&pool)
            .await?;

        Ok((StatusCode::NO_CONTENT, ()))
    } else {
        Err(AppError::NotFound(format!("Category '{}' not found", id)))
    }
}
