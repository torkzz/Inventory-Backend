use crate::{
    auth::AuthenticatedUser,
    errors::AppError,
    models::{
        movement::{CreateMovementRequest, InventoryMovement},
        product::ProductRow,
    },
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct MovementQuery {
    pub product_id: Option<String>,
    pub movement_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_movements(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Query(query): Query<MovementQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = query.limit.unwrap_or(100).min(500);
    let offset = query.offset.unwrap_or(0);

    let rows = sqlx::query_as::<_, InventoryMovement>(
        r#"
        SELECT
            movement_id, product_id, group_id, movement_type, quantity,
            unit, reason, source, device_id, created_by, created_at, server_updated_at
        FROM inventory_movement
        WHERE ($1::text IS NULL OR product_id = $1)
          AND ($2::text IS NULL OR movement_type = $2)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(query.product_id.as_deref())
    .bind(query.movement_type.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await?;

    Ok((StatusCode::OK, Json(rows)))
}

pub async fn get_movement(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query_as::<_, InventoryMovement>(
        r#"
        SELECT
            movement_id, product_id, group_id, movement_type, quantity,
            unit, reason, source, device_id, created_by, created_at, server_updated_at
        FROM inventory_movement
        WHERE movement_id = $1
        "#,
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Movement '{}' not found", id)))?;

    Ok((StatusCode::OK, Json(row)))
}

pub async fn stock_in(
    State(pool): State<PgPool>,
    auth: AuthenticatedUser,
    Json(mut req): Json<CreateMovementRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.movement_type = "STOCK_IN".to_string();
    create_movement_transaction(pool, auth, req).await
}

pub async fn stock_out(
    State(pool): State<PgPool>,
    auth: AuthenticatedUser,
    Json(mut req): Json<CreateMovementRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.movement_type = "STOCK_OUT".to_string();
    create_movement_transaction(pool, auth, req).await
}

pub async fn stock_adjustment(
    State(pool): State<PgPool>,
    auth: AuthenticatedUser,
    Json(mut req): Json<CreateMovementRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.movement_type = "ADJUSTMENT".to_string();
    create_movement_transaction(pool, auth, req).await
}

pub async fn create_movement_generic(
    State(pool): State<PgPool>,
    auth: AuthenticatedUser,
    Json(req): Json<CreateMovementRequest>,
) -> Result<impl IntoResponse, AppError> {
    create_movement_transaction(pool, auth, req).await
}

async fn create_movement_transaction(
    pool: PgPool,
    auth: AuthenticatedUser,
    req: CreateMovementRequest,
) -> Result<impl IntoResponse, AppError> {
    if req.quantity <= 0 && req.movement_type != "ADJUSTMENT" {
        return Err(AppError::Validation(
            "Quantity must be greater than zero".to_string(),
        ));
    }

    let default_group = "default-grocery-group";
    let now = chrono::Utc::now().timestamp_millis();
    let movement_id = req
        .movement_id
        .unwrap_or_else(|| format!("mov-{}", Uuid::new_v4()));
    let created_at = req.created_at.unwrap_or(now);

    let delta = match req.movement_type.to_uppercase().as_str() {
        "STOCK_IN" => req.quantity.abs(),
        "STOCK_OUT" => -req.quantity.abs(),
        "ADJUSTMENT" => req.quantity,
        "INITIAL_BALANCE" => req.quantity,
        _ => return Err(AppError::BadRequest("Invalid movement type".to_string())),
    };

    let mut tx = pool.begin().await?;

    let product_row = sqlx::query(
        "SELECT product_id, quantity, unit FROM product WHERE product_id = $1 FOR UPDATE",
    )
    .bind(&req.product_id)
    .fetch_optional(&mut *tx)
    .await?;

    let (current_quantity, _unit) = match product_row {
        Some(r) => (r.get::<i32, _>("quantity"), r.get::<String, _>("unit")),
        None => {
            return Err(AppError::NotFound(format!(
                "Product '{}' not found",
                req.product_id
            )))
        }
    };

    let new_quantity = current_quantity + delta;
    if new_quantity < 0 {
        return Err(AppError::BadRequest(format!(
            "Insufficient stock: current quantity is {}, cannot reduce by {}",
            current_quantity,
            req.quantity.abs()
        )));
    }

    // Insert Movement Ledger
    sqlx::query(
        r#"
        INSERT INTO inventory_movement (
            movement_id, product_id, group_id, movement_type, quantity,
            unit, reason, source, device_id, created_by, created_at, server_updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
    )
    .bind(&movement_id)
    .bind(&req.product_id)
    .bind(default_group)
    .bind(req.movement_type.to_uppercase())
    .bind(req.quantity)
    .bind(&req.unit)
    .bind(req.reason.as_deref().map(|r| r.trim()))
    .bind(&req.source)
    .bind(req.device_id.as_deref())
    .bind(&auth.0.user_id)
    .bind(created_at)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    // Update Product Stock Balance
    sqlx::query("UPDATE product SET quantity = $1, last_updated = $2, server_updated_at = $2, version = version + 1 WHERE product_id = $3")
        .bind(new_quantity)
        .bind(now)
        .bind(&req.product_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    let movement = InventoryMovement {
        movement_id,
        product_id: req.product_id,
        group_id: default_group.to_string(),
        movement_type: req.movement_type.to_uppercase(),
        quantity: req.quantity,
        unit: req.unit,
        reason: req.reason,
        source: req.source,
        device_id: req.device_id,
        created_by: Some(auth.0.user_id),
        created_at,
        server_updated_at: now,
    };

    Ok((StatusCode::CREATED, Json(movement)))
}

pub async fn get_low_stock(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let products = sqlx::query_as::<_, ProductRow>(
        r#"
        SELECT
            product_id, category_id, group_id, name, brand, description,
            barcode, barcode_format, sku, image_uri, unit, purchase_price,
            selling_price, quantity, minimum_quantity, is_archived, version,
            created_at, last_updated, server_updated_at
        FROM product
        WHERE quantity <= minimum_quantity AND is_archived = FALSE
        ORDER BY quantity ASC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    Ok((StatusCode::OK, Json(products)))
}

pub async fn get_expiring_stock(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let now = chrono::Utc::now().timestamp_millis();
    let thirty_days_later = now + (30 * 24 * 3600 * 1000);

    let rows = sqlx::query(
        r#"
        SELECT b.batch_id, b.product_id, b.quantity, b.price, b.expiration_date, p.name as product_name
        FROM stock_batch b
        JOIN product p ON b.product_id = p.product_id
        WHERE b.expiration_date IS NOT NULL
          AND b.expiration_date <= $1
        ORDER BY b.expiration_date ASC
        "#,
    )
    .bind(thirty_days_later)
    .fetch_all(&pool)
    .await?;

    let list: Vec<_> = rows
        .into_iter()
        .map(|r| {
            json!({
                "batchId": r.get::<String, _>("batch_id"),
                "productId": r.get::<String, _>("product_id"),
                "productName": r.get::<String, _>("product_name"),
                "quantity": r.get::<i32, _>("quantity"),
                "price": r.get::<f64, _>("price"),
                "expirationDate": r.get::<Option<i64>, _>("expiration_date")
            })
        })
        .collect();

    Ok((StatusCode::OK, Json(list)))
}
