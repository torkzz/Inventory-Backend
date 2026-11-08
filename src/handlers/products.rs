use crate::{
    auth::AuthenticatedUser,
    errors::AppError,
    models::{
        category::CategoryResponse,
        product::{ProductRequest, ProductResponse, ProductRow},
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
pub struct ProductQuery {
    pub query: Option<String>,
    pub category_id: Option<String>,
    pub barcode: Option<String>,
    pub is_archived: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_products(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Query(query): Query<ProductQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = query.limit.unwrap_or(100).min(500);
    let offset = query.offset.unwrap_or(0);
    let is_archived = query.is_archived.unwrap_or(false);

    let rows = sqlx::query_as::<_, ProductRow>(
        r#"
        SELECT
            p.product_id, p.category_id, p.group_id, p.name, p.brand, p.description,
            p.barcode, p.barcode_format, p.sku, p.image_uri, p.unit, p.purchase_price,
            p.selling_price, p.quantity, p.minimum_quantity, p.is_archived, p.version,
            p.created_at, p.last_updated, p.server_updated_at
        FROM product p
        WHERE p.is_archived = $1
          AND ($2::text IS NULL OR p.name ILIKE ('%' || $2 || '%') OR p.barcode ILIKE ('%' || $2 || '%') OR p.brand ILIKE ('%' || $2 || '%'))
          AND ($3::text IS NULL OR p.category_id = $3)
          AND ($4::text IS NULL OR p.barcode = $4)
        ORDER BY p.last_updated DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(is_archived)
    .bind(query.query.as_deref())
    .bind(query.category_id.as_deref())
    .bind(query.barcode.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await?;

    let mut response_list = Vec::with_capacity(rows.len());
    for row in rows {
        let category = if let Some(ref cat_id) = row.category_id {
            sqlx::query_as::<_, crate::models::category::Category>(
                "SELECT category_id, group_id, name, description, created_at, last_updated, server_updated_at FROM category WHERE category_id = $1",
            )
            .bind(cat_id)
            .fetch_optional(&pool)
            .await?
            .map(CategoryResponse::from)
        } else {
            None
        };

        response_list.push(ProductResponse {
            product_id: row.product_id,
            group_id: row.group_id,
            category,
            name: row.name,
            brand: row.brand,
            description: row.description,
            barcode: row.barcode,
            barcode_format: row.barcode_format,
            sku: row.sku,
            image_uri: row.image_uri,
            unit: row.unit,
            purchase_price: row.purchase_price,
            selling_price: row.selling_price,
            quantity: row.quantity,
            minimum_quantity: row.minimum_quantity,
            is_archived: row.is_archived,
            version: row.version,
            batches: vec![],
            created_at: row.created_at,
            last_updated: row.last_updated,
        });
    }

    Ok((StatusCode::OK, Json(response_list)))
}

pub async fn get_product(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query_as::<_, ProductRow>(
        r#"
        SELECT
            product_id, category_id, group_id, name, brand, description,
            barcode, barcode_format, sku, image_uri, unit, purchase_price,
            selling_price, quantity, minimum_quantity, is_archived, version,
            created_at, last_updated, server_updated_at
        FROM product
        WHERE product_id = $1
        "#,
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Product '{}' not found", id)))?;

    let category = if let Some(ref cat_id) = row.category_id {
        sqlx::query_as::<_, crate::models::category::Category>(
            "SELECT category_id, group_id, name, description, created_at, last_updated, server_updated_at FROM category WHERE category_id = $1",
        )
        .bind(cat_id)
        .fetch_optional(&pool)
        .await?
        .map(CategoryResponse::from)
    } else {
        None
    };

    Ok((
        StatusCode::OK,
        Json(ProductResponse {
            product_id: row.product_id,
            group_id: row.group_id,
            category,
            name: row.name,
            brand: row.brand,
            description: row.description,
            barcode: row.barcode,
            barcode_format: row.barcode_format,
            sku: row.sku,
            image_uri: row.image_uri,
            unit: row.unit,
            purchase_price: row.purchase_price,
            selling_price: row.selling_price,
            quantity: row.quantity,
            minimum_quantity: row.minimum_quantity,
            is_archived: row.is_archived,
            version: row.version,
            batches: vec![],
            created_at: row.created_at,
            last_updated: row.last_updated,
        }),
    ))
}

pub async fn get_product_by_barcode(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Path(barcode): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let clean = barcode.trim();
    let row = sqlx::query_as::<_, ProductRow>(
        r#"
        SELECT
            product_id, category_id, group_id, name, brand, description,
            barcode, barcode_format, sku, image_uri, unit, purchase_price,
            selling_price, quantity, minimum_quantity, is_archived, version,
            created_at, last_updated, server_updated_at
        FROM product
        WHERE barcode = $1
        LIMIT 1
        "#,
    )
    .bind(clean)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("No product found with barcode '{}'", clean)))?;

    let category = if let Some(ref cat_id) = row.category_id {
        sqlx::query_as::<_, crate::models::category::Category>(
            "SELECT category_id, group_id, name, description, created_at, last_updated, server_updated_at FROM category WHERE category_id = $1",
        )
        .bind(cat_id)
        .fetch_optional(&pool)
        .await?
        .map(CategoryResponse::from)
    } else {
        None
    };

    Ok((
        StatusCode::OK,
        Json(ProductResponse {
            product_id: row.product_id,
            group_id: row.group_id,
            category,
            name: row.name,
            brand: row.brand,
            description: row.description,
            barcode: row.barcode,
            barcode_format: row.barcode_format,
            sku: row.sku,
            image_uri: row.image_uri,
            unit: row.unit,
            purchase_price: row.purchase_price,
            selling_price: row.selling_price,
            quantity: row.quantity,
            minimum_quantity: row.minimum_quantity,
            is_archived: row.is_archived,
            version: row.version,
            batches: vec![],
            created_at: row.created_at,
            last_updated: row.last_updated,
        }),
    ))
}

pub async fn create_product(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Json(payload): Json<ProductRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Product name is required".to_string()));
    }

    let now = chrono::Utc::now().timestamp_millis();
    let product_id = payload
        .product_id
        .unwrap_or_else(|| format!("product-{}", Uuid::new_v4()));
    let default_group = "default-grocery-group";
    let clean_barcode = payload
        .barcode
        .as_deref()
        .map(|b| b.trim())
        .filter(|b| !b.is_empty());
    let clean_sku = payload
        .sku
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    // Check barcode duplicate
    if let Some(bc) = clean_barcode {
        let exists =
            sqlx::query("SELECT 1 as ex FROM product WHERE barcode = $1 AND group_id = $2")
                .bind(bc)
                .bind(default_group)
                .fetch_optional(&pool)
                .await?;

        if exists.is_some() {
            return Err(AppError::Conflict("Barcode already exists".to_string()));
        }
    }

    let created_at = payload.created_at.unwrap_or(now);
    let last_updated = payload.last_updated.unwrap_or(now);

    sqlx::query(
        r#"
        INSERT INTO product (
            product_id, category_id, group_id, name, brand, description,
            barcode, barcode_format, sku, image_uri, unit, purchase_price,
            selling_price, quantity, minimum_quantity, is_archived, version,
            created_at, last_updated, server_updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20
        )
        "#
    )
    .bind(&product_id)
    .bind(payload.category_id.as_deref())
    .bind(default_group)
    .bind(payload.name.trim())
    .bind(payload.brand.as_deref().map(|b| b.trim()).filter(|b| !b.is_empty()))
    .bind(payload.description.as_deref().map(|d| d.trim()))
    .bind(clean_barcode)
    .bind(payload.barcode_format.as_deref())
    .bind(clean_sku)
    .bind(payload.image_uri.as_deref())
    .bind(&payload.unit)
    .bind(payload.purchase_price)
    .bind(payload.selling_price)
    .bind(payload.quantity)
    .bind(payload.minimum_quantity)
    .bind(payload.is_archived)
    .bind(payload.version)
    .bind(created_at)
    .bind(last_updated)
    .bind(now)
    .execute(&pool)
    .await?;

    // If initial stock quantity > 0, generate INITIAL_BALANCE ledger movement
    if payload.quantity > 0 {
        let mov_id = format!("mov-init-{}", product_id);
        let _ = sqlx::query(
            r#"
            INSERT INTO inventory_movement (
                movement_id, product_id, group_id, movement_type, quantity,
                unit, reason, source, created_at, server_updated_at
            ) VALUES ($1, $2, $3, 'INITIAL_BALANCE', 0, $4, 'Initial stock entry', 'SYSTEM', $5, $5)
            ON CONFLICT (movement_id) DO NOTHING
            "#,
        )
        .bind(&mov_id)
        .bind(&product_id)
        .bind(default_group)
        .bind(&payload.unit)
        .bind(now)
        .execute(&pool)
        .await;
    }

    let category = if let Some(ref cat_id) = payload.category_id {
        sqlx::query_as::<_, crate::models::category::Category>(
            "SELECT category_id, group_id, name, description, created_at, last_updated, server_updated_at FROM category WHERE category_id = $1",
        )
        .bind(cat_id)
        .fetch_optional(&pool)
        .await?
        .map(CategoryResponse::from)
    } else {
        None
    };

    let response = ProductResponse {
        product_id,
        group_id: default_group.to_string(),
        category,
        name: payload.name.trim().to_string(),
        brand: payload.brand,
        description: payload.description,
        barcode: clean_barcode.map(|s| s.to_string()),
        barcode_format: payload.barcode_format,
        sku: clean_sku.map(|s| s.to_string()),
        image_uri: payload.image_uri,
        unit: payload.unit,
        purchase_price: payload.purchase_price,
        selling_price: payload.selling_price,
        quantity: payload.quantity,
        minimum_quantity: payload.minimum_quantity,
        is_archived: payload.is_archived,
        version: payload.version,
        batches: vec![],
        created_at,
        last_updated,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_product(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<ProductRequest>,
) -> Result<impl IntoResponse, AppError> {
    let now = chrono::Utc::now().timestamp_millis();
    let clean_barcode = payload
        .barcode
        .as_deref()
        .map(|b| b.trim())
        .filter(|b| !b.is_empty());
    let clean_sku = payload
        .sku
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    let res = sqlx::query(
        r#"
        UPDATE product
        SET
            category_id = $1,
            name = $2,
            brand = $3,
            description = $4,
            barcode = $5,
            barcode_format = $6,
            sku = $7,
            image_uri = $8,
            unit = $9,
            purchase_price = $10,
            selling_price = $11,
            quantity = $12,
            minimum_quantity = $13,
            is_archived = $14,
            version = version + 1,
            last_updated = $15,
            server_updated_at = $15
        WHERE product_id = $16
        "#,
    )
    .bind(payload.category_id.as_deref())
    .bind(payload.name.trim())
    .bind(
        payload
            .brand
            .as_deref()
            .map(|b| b.trim())
            .filter(|b| !b.is_empty()),
    )
    .bind(payload.description.as_deref().map(|d| d.trim()))
    .bind(clean_barcode)
    .bind(payload.barcode_format.as_deref())
    .bind(clean_sku)
    .bind(payload.image_uri.as_deref())
    .bind(&payload.unit)
    .bind(payload.purchase_price)
    .bind(payload.selling_price)
    .bind(payload.quantity)
    .bind(payload.minimum_quantity)
    .bind(payload.is_archived)
    .bind(now)
    .bind(&id)
    .execute(&pool)
    .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Product '{}' not found", id)));
    }

    let updated = sqlx::query_as::<_, ProductRow>(
        r#"
        SELECT
            product_id, category_id, group_id, name, brand, description,
            barcode, barcode_format, sku, image_uri, unit, purchase_price,
            selling_price, quantity, minimum_quantity, is_archived, version,
            created_at, last_updated, server_updated_at
        FROM product WHERE product_id = $1
        "#,
    )
    .bind(&id)
    .fetch_one(&pool)
    .await?;

    let category = if let Some(ref cat_id) = updated.category_id {
        sqlx::query_as::<_, crate::models::category::Category>(
            "SELECT category_id, group_id, name, description, created_at, last_updated, server_updated_at FROM category WHERE category_id = $1",
        )
        .bind(cat_id)
        .fetch_optional(&pool)
        .await?
        .map(CategoryResponse::from)
    } else {
        None
    };

    Ok((
        StatusCode::OK,
        Json(ProductResponse {
            product_id: updated.product_id,
            group_id: updated.group_id,
            category,
            name: updated.name,
            brand: updated.brand,
            description: updated.description,
            barcode: updated.barcode,
            barcode_format: updated.barcode_format,
            sku: updated.sku,
            image_uri: updated.image_uri,
            unit: updated.unit,
            purchase_price: updated.purchase_price,
            selling_price: updated.selling_price,
            quantity: updated.quantity,
            minimum_quantity: updated.minimum_quantity,
            is_archived: updated.is_archived,
            version: updated.version,
            batches: vec![],
            created_at: updated.created_at,
            last_updated: updated.last_updated,
        }),
    ))
}

pub async fn delete_product(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let now = chrono::Utc::now().timestamp_millis();

    let row = sqlx::query("SELECT group_id FROM product WHERE product_id = $1")
        .bind(&id)
        .fetch_optional(&pool)
        .await?;

    if let Some(r) = row {
        let group_id: String = r.get("group_id");

        sqlx::query(
            "INSERT INTO deleted (entity_id, group_id, entity_type, deleted_at) VALUES ($1, $2, 'PRODUCT', $3) ON CONFLICT (entity_id, entity_type) DO UPDATE SET deleted_at = $3"
        )
        .bind(&id)
        .bind(&group_id)
        .bind(now)
        .execute(&pool)
        .await?;

        sqlx::query("DELETE FROM product WHERE product_id = $1")
            .bind(&id)
            .execute(&pool)
            .await?;

        Ok((StatusCode::NO_CONTENT, ()))
    } else {
        Err(AppError::NotFound(format!("Product '{}' not found", id)))
    }
}

pub async fn archive_product(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let now = chrono::Utc::now().timestamp_millis();
    let row = sqlx::query("SELECT is_archived FROM product WHERE product_id = $1")
        .bind(&id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product '{}' not found", id)))?;

    let current_archived: bool = row.get("is_archived");
    let new_archived = !current_archived;

    sqlx::query(
        "UPDATE product SET is_archived = $1, version = version + 1, last_updated = $2, server_updated_at = $2 WHERE product_id = $3"
    )
    .bind(new_archived)
    .bind(now)
    .bind(&id)
    .execute(&pool)
    .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "productId": id,
            "isArchived": new_archived
        })),
    ))
}
