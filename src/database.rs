use crate::errors::AppError;
use sqlx::{postgres::PgPoolOptions, Executor, PgPool};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
        .map_err(AppError::Database)?;

    initialize_schema(&pool).await?;

    Ok(pool)
}

pub async fn initialize_schema(pool: &PgPool) -> Result<(), AppError> {
    let statements = [
        r#"
        CREATE TABLE IF NOT EXISTS groups (
            group_id VARCHAR(64) PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            address TEXT,
            total_members INT NOT NULL DEFAULT 1,
            total_products INT NOT NULL DEFAULT 0,
            created_at BIGINT NOT NULL,
            last_updated BIGINT NOT NULL
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS users (
            user_id VARCHAR(64) PRIMARY KEY,
            email VARCHAR(255) NOT NULL UNIQUE,
            pass_hash VARCHAR(255) NOT NULL,
            name VARCHAR(255) NOT NULL,
            avatar_url TEXT,
            phone_number VARCHAR(64),
            job_title VARCHAR(128),
            locale VARCHAR(32),
            time_zone VARCHAR(64),
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at BIGINT NOT NULL,
            last_updated BIGINT NOT NULL
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS user_groups (
            user_id VARCHAR(64) REFERENCES users(user_id) ON DELETE CASCADE,
            group_id VARCHAR(64) REFERENCES groups(group_id) ON DELETE CASCADE,
            role VARCHAR(32) NOT NULL DEFAULT 'OWNER',
            status VARCHAR(32) NOT NULL DEFAULT 'ACTIVE',
            joined_at BIGINT NOT NULL,
            PRIMARY KEY (user_id, group_id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS refresh_tokens (
            user_id VARCHAR(64) REFERENCES users(user_id) ON DELETE CASCADE,
            token VARCHAR(255) PRIMARY KEY,
            expires_at BIGINT NOT NULL,
            is_revoked BOOLEAN NOT NULL DEFAULT FALSE
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS category (
            category_id VARCHAR(64) PRIMARY KEY,
            group_id VARCHAR(64) REFERENCES groups(group_id) ON DELETE CASCADE,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            created_at BIGINT NOT NULL,
            last_updated BIGINT NOT NULL,
            server_updated_at BIGINT NOT NULL DEFAULT 0,
            CONSTRAINT category_group_id_name_unique UNIQUE (group_id, name)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS location (
            location_id VARCHAR(64) PRIMARY KEY,
            group_id VARCHAR(64) REFERENCES groups(group_id) ON DELETE CASCADE,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            location_barcode VARCHAR(64),
            created_at BIGINT NOT NULL,
            last_updated BIGINT NOT NULL,
            server_updated_at BIGINT NOT NULL DEFAULT 0
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS product (
            product_id VARCHAR(64) PRIMARY KEY,
            category_id VARCHAR(64) REFERENCES category(category_id) ON DELETE SET NULL,
            group_id VARCHAR(64) REFERENCES groups(group_id) ON DELETE CASCADE,
            name VARCHAR(255) NOT NULL,
            brand VARCHAR(255),
            description TEXT,
            barcode VARCHAR(64),
            barcode_format VARCHAR(64),
            sku VARCHAR(128),
            image_uri TEXT,
            unit VARCHAR(64) NOT NULL DEFAULT 'pcs',
            purchase_price DOUBLE PRECISION NOT NULL DEFAULT 0.0,
            selling_price DOUBLE PRECISION NOT NULL DEFAULT 0.0,
            quantity INT NOT NULL DEFAULT 0,
            minimum_quantity INT NOT NULL DEFAULT 0,
            is_archived BOOLEAN NOT NULL DEFAULT FALSE,
            version BIGINT NOT NULL DEFAULT 1,
            created_at BIGINT NOT NULL,
            last_updated BIGINT NOT NULL,
            server_updated_at BIGINT NOT NULL DEFAULT 0,
            CONSTRAINT product_group_id_barcode_unique UNIQUE (group_id, barcode),
            CONSTRAINT product_group_id_sku_unique UNIQUE (group_id, sku)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS stock_batch (
            batch_id VARCHAR(64) PRIMARY KEY,
            product_id VARCHAR(64) REFERENCES product(product_id) ON DELETE CASCADE,
            location_id VARCHAR(64) REFERENCES location(location_id) ON DELETE RESTRICT,
            group_id VARCHAR(64) REFERENCES groups(group_id) ON DELETE CASCADE,
            quantity INT NOT NULL,
            price DOUBLE PRECISION NOT NULL DEFAULT 0.0,
            expiration_date BIGINT,
            supplier TEXT,
            created_at BIGINT NOT NULL,
            last_updated BIGINT NOT NULL,
            server_updated_at BIGINT NOT NULL DEFAULT 0
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS inventory_movement (
            movement_id VARCHAR(64) PRIMARY KEY,
            product_id VARCHAR(64) REFERENCES product(product_id) ON DELETE CASCADE,
            group_id VARCHAR(64) REFERENCES groups(group_id) ON DELETE CASCADE,
            movement_type VARCHAR(32) NOT NULL,
            quantity INT NOT NULL,
            unit VARCHAR(64) NOT NULL DEFAULT 'pcs',
            reason VARCHAR(255),
            source VARCHAR(32) NOT NULL DEFAULT 'SYSTEM',
            device_id VARCHAR(64),
            created_by VARCHAR(64),
            created_at BIGINT NOT NULL,
            server_updated_at BIGINT NOT NULL DEFAULT 0
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS deleted (
            entity_id VARCHAR(64) NOT NULL,
            group_id VARCHAR(64) NOT NULL,
            entity_type VARCHAR(32) NOT NULL,
            deleted_at BIGINT NOT NULL,
            PRIMARY KEY (entity_id, group_id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS device_registration (
            device_id VARCHAR(64) PRIMARY KEY,
            device_name VARCHAR(255) NOT NULL,
            device_model VARCHAR(255),
            group_id VARCHAR(64) REFERENCES groups(group_id) ON DELETE CASCADE,
            last_sync_at BIGINT NOT NULL DEFAULT 0,
            registered_at BIGINT NOT NULL
        )
        "#,
    ];

    for stmt in statements {
        pool.execute(stmt).await.map_err(AppError::Database)?;
    }

    // Safe column migrations
    let _ = pool
        .execute("ALTER TABLE product ALTER COLUMN category_id DROP NOT NULL;")
        .await;
    let _ = pool
        .execute("ALTER TABLE product ADD COLUMN IF NOT EXISTS brand VARCHAR(255);")
        .await;
    let _ = pool
        .execute("ALTER TABLE product ADD COLUMN IF NOT EXISTS unit VARCHAR(64) DEFAULT 'pcs';")
        .await;
    let _ = pool.execute("ALTER TABLE product ADD COLUMN IF NOT EXISTS purchase_price DOUBLE PRECISION DEFAULT 0.0;").await;
    let _ = pool.execute("ALTER TABLE product ADD COLUMN IF NOT EXISTS selling_price DOUBLE PRECISION DEFAULT 0.0;").await;
    let _ = pool
        .execute("ALTER TABLE product ADD COLUMN IF NOT EXISTS quantity INT DEFAULT 0;")
        .await;
    let _ = pool
        .execute("ALTER TABLE product ADD COLUMN IF NOT EXISTS is_archived BOOLEAN DEFAULT FALSE;")
        .await;
    let _ = pool
        .execute("ALTER TABLE product ADD COLUMN IF NOT EXISTS version BIGINT DEFAULT 1;")
        .await;

    Ok(())
}
