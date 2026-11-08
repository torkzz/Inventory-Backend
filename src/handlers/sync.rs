use crate::{
    auth::AuthenticatedUser,
    errors::AppError,
    models::{
        category::CategoryResponse,
        dashboard::DeviceRegistrationDto,
        movement::InventoryMovement,
        sync::{
            LocationDto, SyncPayloadDto, SyncProductDto, SyncPullResponseDto, SyncPushResponseDto,
            SyncStockBatchDto,
        },
    },
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::{PgPool, Row};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
pub struct PullQuery {
    #[serde(rename = "updatedAfter", default)]
    pub updated_after: i64,
}

pub async fn push_sync(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Path(group_id): Path<String>,
    Json(payload): Json<SyncPayloadDto>,
) -> Result<impl IntoResponse, AppError> {
    let now = chrono::Utc::now().timestamp_millis();
    let mut tx = pool.begin().await?;

    // 1. Process deletions
    for cat_id in &payload.deleted_categories {
        sqlx::query("INSERT INTO deleted (entity_id, group_id, entity_type, deleted_at) VALUES ($1, $2, 'CATEGORY', $3) ON CONFLICT (entity_id, entity_type) DO UPDATE SET deleted_at = $3")
            .bind(cat_id).bind(&group_id).bind(now).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM category WHERE category_id = $1 AND group_id = $2")
            .bind(cat_id)
            .bind(&group_id)
            .execute(&mut *tx)
            .await?;
    }
    for prod_id in &payload.deleted_product {
        sqlx::query("INSERT INTO deleted (entity_id, group_id, entity_type, deleted_at) VALUES ($1, $2, 'PRODUCT', $3) ON CONFLICT (entity_id, entity_type) DO UPDATE SET deleted_at = $3")
            .bind(prod_id).bind(&group_id).bind(now).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM product WHERE product_id = $1 AND group_id = $2")
            .bind(prod_id)
            .bind(&group_id)
            .execute(&mut *tx)
            .await?;
    }
    for loc_id in &payload.deleted_locations {
        sqlx::query("INSERT INTO deleted (entity_id, group_id, entity_type, deleted_at) VALUES ($1, $2, 'LOCATION', $3) ON CONFLICT (entity_id, entity_type) DO UPDATE SET deleted_at = $3")
            .bind(loc_id).bind(&group_id).bind(now).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM location WHERE location_id = $1 AND group_id = $2")
            .bind(loc_id)
            .bind(&group_id)
            .execute(&mut *tx)
            .await?;
    }
    for batch_id in &payload.deleted_batches {
        sqlx::query("INSERT INTO deleted (entity_id, group_id, entity_type, deleted_at) VALUES ($1, $2, 'BATCH', $3) ON CONFLICT (entity_id, entity_type) DO UPDATE SET deleted_at = $3")
            .bind(batch_id).bind(&group_id).bind(now).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM stock_batch WHERE batch_id = $1 AND group_id = $2")
            .bind(batch_id)
            .bind(&group_id)
            .execute(&mut *tx)
            .await?;
    }

    // 2. Categories
    let all_categories = payload
        .created_categories
        .into_iter()
        .chain(payload.updated_categories);
    for cat in all_categories {
        if let Some(cat_id) = cat.category_id {
            sqlx::query(
                r#"
                INSERT INTO category (category_id, group_id, name, description, created_at, last_updated, server_updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (category_id) DO UPDATE
                SET name = $3, description = $4, last_updated = $6, server_updated_at = $7
                "#
            )
            .bind(cat_id)
            .bind(&group_id)
            .bind(cat.name.trim())
            .bind(cat.description.as_deref().map(|d| d.trim()))
            .bind(cat.created_at.unwrap_or(now))
            .bind(cat.last_updated.unwrap_or(now))
            .bind(now)
            .execute(&mut *tx)
            .await?;
        }
    }

    // 3. Locations
    let all_locations = payload
        .created_locations
        .into_iter()
        .chain(payload.updated_locations);
    for loc in all_locations {
        sqlx::query(
            r#"
            INSERT INTO location (location_id, group_id, name, description, location_barcode, created_at, last_updated, server_updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (location_id) DO UPDATE
            SET name = $3, description = $4, location_barcode = $5, last_updated = $7, server_updated_at = $8
            "#
        )
        .bind(loc.location_id)
        .bind(&group_id)
        .bind(loc.name.trim())
        .bind(loc.description.as_deref().map(|d| d.trim()))
        .bind(loc.location_barcode.as_deref())
        .bind(loc.created_at)
        .bind(loc.last_updated)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    // 4. Products
    let all_products = payload
        .created_product
        .into_iter()
        .chain(payload.updated_product);
    for prod in all_products {
        let clean_barcode = prod
            .barcode
            .as_deref()
            .map(|b| b.trim())
            .filter(|b| !b.is_empty());
        let clean_sku = prod
            .sku
            .as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());

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
            ON CONFLICT (product_id) DO UPDATE SET
                category_id = EXCLUDED.category_id,
                name = EXCLUDED.name,
                brand = EXCLUDED.brand,
                description = EXCLUDED.description,
                barcode = EXCLUDED.barcode,
                barcode_format = EXCLUDED.barcode_format,
                sku = EXCLUDED.sku,
                image_uri = COALESCE(EXCLUDED.image_uri, product.image_uri),
                unit = EXCLUDED.unit,
                purchase_price = EXCLUDED.purchase_price,
                selling_price = EXCLUDED.selling_price,
                quantity = EXCLUDED.quantity,
                minimum_quantity = EXCLUDED.minimum_quantity,
                is_archived = EXCLUDED.is_archived,
                version = EXCLUDED.version + 1,
                last_updated = EXCLUDED.last_updated,
                server_updated_at = EXCLUDED.server_updated_at
            WHERE product.last_updated <= EXCLUDED.last_updated
            "#
        )
        .bind(prod.product_id)
        .bind(prod.category_id.as_deref())
        .bind(&group_id)
        .bind(prod.name.trim())
        .bind(prod.brand.as_deref().map(|b| b.trim()).filter(|b| !b.is_empty()))
        .bind(prod.description.as_deref().map(|d| d.trim()))
        .bind(clean_barcode)
        .bind(prod.barcode_format.as_deref())
        .bind(clean_sku)
        .bind(prod.image_uri.as_deref())
        .bind(prod.unit)
        .bind(prod.purchase_price)
        .bind(prod.selling_price)
        .bind(prod.quantity)
        .bind(prod.minimum_quantity)
        .bind(prod.is_archived)
        .bind(prod.version)
        .bind(prod.created_at)
        .bind(prod.last_updated)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    // 5. Batches
    let all_batches = payload
        .created_batches
        .into_iter()
        .chain(payload.updated_batches);
    for b in all_batches {
        sqlx::query(
            r#"
            INSERT INTO stock_batch (
                batch_id, product_id, location_id, group_id, quantity, price,
                expiration_date, supplier, created_at, last_updated, server_updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (batch_id) DO UPDATE SET
                quantity = EXCLUDED.quantity,
                price = EXCLUDED.price,
                expiration_date = EXCLUDED.expiration_date,
                supplier = EXCLUDED.supplier,
                last_updated = EXCLUDED.last_updated,
                server_updated_at = EXCLUDED.server_updated_at
            "#,
        )
        .bind(b.batch_id)
        .bind(b.product_id)
        .bind(b.location_id)
        .bind(&group_id)
        .bind(b.quantity)
        .bind(b.price)
        .bind(b.expiration_date)
        .bind(b.supplier.as_deref())
        .bind(b.created_at)
        .bind(b.last_updated)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    // 6. Movements (idempotent ledger insert)
    for m in payload.movements {
        let inserted = sqlx::query(
            r#"
            INSERT INTO inventory_movement (
                movement_id, product_id, group_id, movement_type, quantity,
                unit, reason, source, device_id, created_by, created_at, server_updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (movement_id) DO NOTHING
            "#,
        )
        .bind(&m.movement_id)
        .bind(&m.product_id)
        .bind(&group_id)
        .bind(&m.movement_type)
        .bind(m.quantity)
        .bind(&m.unit)
        .bind(m.reason.as_deref())
        .bind(&m.source)
        .bind(m.device_id.as_deref())
        .bind(m.created_by.as_deref())
        .bind(m.created_at)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        if inserted.rows_affected() > 0 && m.quantity != 0 {
            let delta = match m.movement_type.to_uppercase().as_str() {
                "STOCK_IN" => m.quantity.abs(),
                "STOCK_OUT" => -m.quantity.abs(),
                "ADJUSTMENT" => m.quantity,
                _ => 0,
            };
            if delta != 0 {
                sqlx::query(
                    "UPDATE product SET quantity = quantity + $1, last_updated = $2, server_updated_at = $2 WHERE product_id = $3 AND group_id = $4"
                )
                .bind(delta)
                .bind(now)
                .bind(&m.product_id)
                .bind(&group_id)
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    tx.commit().await?;

    Ok((
        StatusCode::OK,
        Json(SyncPushResponseDto {
            group_id,
            success: true,
            message: "Sync Success".to_string(),
            server_time_stamp: now,
        }),
    ))
}

pub async fn pull_sync(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Path(group_id): Path<String>,
    Query(query): Query<PullQuery>,
) -> Result<impl IntoResponse, AppError> {
    let since = query.updated_after;
    let now = chrono::Utc::now().timestamp_millis();

    let categories = sqlx::query_as::<_, crate::models::category::Category>(
        "SELECT category_id, group_id, name, description, created_at, last_updated, server_updated_at FROM category WHERE server_updated_at > $1 AND group_id = $2",
    )
    .bind(since)
    .bind(&group_id)
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(CategoryResponse::from)
    .collect();

    let locations = sqlx::query_as::<_, LocationDto>(
        "SELECT location_id, group_id, name, description, location_barcode, created_at, last_updated FROM location WHERE server_updated_at > $1 AND group_id = $2",
    )
    .bind(since)
    .bind(&group_id)
    .fetch_all(&pool)
    .await?;

    let products = sqlx::query_as::<_, SyncProductDto>(
        r#"
        SELECT
            product_id, category_id, group_id, name, brand, description,
            barcode, barcode_format, sku, image_uri, unit, purchase_price,
            selling_price, quantity, minimum_quantity, is_archived, version,
            created_at, last_updated
        FROM product
        WHERE server_updated_at > $1 AND group_id = $2
        "#,
    )
    .bind(since)
    .bind(&group_id)
    .fetch_all(&pool)
    .await?;

    let batches = sqlx::query_as::<_, SyncStockBatchDto>(
        r#"
        SELECT
            batch_id, product_id, location_id, group_id, quantity, price,
            expiration_date, supplier, created_at, last_updated
        FROM stock_batch
        WHERE server_updated_at > $1 AND group_id = $2
        "#,
    )
    .bind(since)
    .bind(&group_id)
    .fetch_all(&pool)
    .await?;

    let movements = sqlx::query_as::<_, InventoryMovement>(
        r#"
        SELECT
            movement_id, product_id, group_id, movement_type, quantity,
            unit, reason, source, device_id, created_by, created_at, server_updated_at
        FROM inventory_movement
        WHERE server_updated_at > $1 AND group_id = $2
        "#,
    )
    .bind(since)
    .bind(&group_id)
    .fetch_all(&pool)
    .await?;

    let deleted_rows = sqlx::query(
        "SELECT entity_id, entity_type FROM deleted WHERE deleted_at > $1 AND group_id = $2",
    )
    .bind(since)
    .bind(&group_id)
    .fetch_all(&pool)
    .await?;

    let mut deleted_categories = Vec::new();
    let mut deleted_locations = Vec::new();
    let mut deleted_products = Vec::new();
    let mut deleted_batches = Vec::new();

    for row in deleted_rows {
        let entity_id: String = row.get("entity_id");
        let entity_type: String = row.get("entity_type");
        match entity_type.as_str() {
            "CATEGORY" => deleted_categories.push(entity_id),
            "LOCATION" => deleted_locations.push(entity_id),
            "PRODUCT" => deleted_products.push(entity_id),
            "BATCH" => deleted_batches.push(entity_id),
            _ => {}
        }
    }

    Ok((
        StatusCode::OK,
        Json(SyncPullResponseDto {
            categories,
            locations,
            products,
            batches,
            movements,
            deleted_categories,
            deleted_locations,
            deleted_products,
            deleted_batches,
            group_id,
            server_time_stamp: now,
        }),
    ))
}

pub async fn register_device(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
    Json(payload): Json<DeviceRegistrationDto>,
) -> Result<impl IntoResponse, AppError> {
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        r#"
        INSERT INTO device_registration (device_id, device_name, device_model, group_id, last_sync_at, registered_at)
        VALUES ($1, $2, $3, $4, $5, $5)
        ON CONFLICT (device_id) DO UPDATE
        SET device_name = $2, device_model = $3, last_sync_at = $5
        "#
    )
    .bind(&payload.device_id)
    .bind(&payload.device_name)
    .bind(payload.device_model.as_deref())
    .bind(&payload.group_id)
    .bind(now)
    .execute(&pool)
    .await?;

    Ok((
        StatusCode::OK,
        Json(json!({"registered": true, "deviceId": payload.device_id})),
    ))
}

pub async fn list_devices(
    State(pool): State<PgPool>,
    _auth: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let devices = sqlx::query_as::<_, DeviceRegistrationDto>(
        "SELECT device_id, device_name, device_model, group_id, last_sync_at, registered_at FROM device_registration ORDER BY last_sync_at DESC"
    )
    .fetch_all(&pool)
    .await?;

    Ok((StatusCode::OK, Json(devices)))
}

pub async fn sync_ws_handler(
    ws: WebSocketUpgrade,
    Path(_group_id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(|mut socket: WebSocket| async move {
        loop {
            if socket
                .send(Message::Text("SYNC_PING".to_string()))
                .await
                .is_err()
            {
                break;
            }
            sleep(Duration::from_secs(20)).await;
        }
    })
}
