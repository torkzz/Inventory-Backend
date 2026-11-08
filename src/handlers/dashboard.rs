use crate::{
    auth::AuthenticatedUser,
    errors::AppError,
    models::{dashboard::DashboardSummaryDto, movement::InventoryMovement},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use sqlx::{PgPool, Row};

pub async fn get_dashboard_summary(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let total_products: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM product WHERE is_archived = FALSE")
            .fetch_one(&pool)
            .await?;

    let total_categories: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM category")
        .fetch_one(&pool)
        .await?;

    let low_stock_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM product WHERE quantity <= minimum_quantity AND is_archived = FALSE",
    )
    .fetch_one(&pool)
    .await?;

    let total_stock_value: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(quantity * CASE WHEN purchase_price > 0 THEN purchase_price ELSE selling_price END), 0.0) FROM product WHERE is_archived = FALSE"
    )
    .fetch_one(&pool)
    .await?;

    let recent_movements = sqlx::query_as::<_, InventoryMovement>(
        r#"
        SELECT
            movement_id, product_id, group_id, movement_type, quantity,
            unit, reason, source, device_id, created_by, created_at, server_updated_at
        FROM inventory_movement
        ORDER BY created_at DESC
        LIMIT 15
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let summary = DashboardSummaryDto {
        total_products,
        total_categories,
        low_stock_count,
        total_stock_value,
        recent_movements,
    };

    Ok((StatusCode::OK, Json(summary)))
}

pub async fn get_category_summary(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT c.category_id, c.name, COUNT(p.product_id) as product_count, COALESCE(SUM(p.quantity), 0) as total_quantity
        FROM category c
        LEFT JOIN product p ON c.category_id = p.category_id AND p.is_archived = FALSE
        GROUP BY c.category_id, c.name
        ORDER BY product_count DESC
        "#
    )
    .fetch_all(&pool)
    .await?;

    let list: Vec<_> = rows
        .into_iter()
        .map(|r| {
            json!({
                "categoryId": r.get::<String, _>("category_id"),
                "name": r.get::<String, _>("name"),
                "productCount": r.get::<i64, _>("product_count"),
                "totalQuantity": r.get::<i64, _>("total_quantity")
            })
        })
        .collect();

    Ok((StatusCode::OK, Json(list)))
}
