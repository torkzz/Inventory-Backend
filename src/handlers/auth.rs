use crate::{
    auth::{generate_jwt, hash_password, hash_token_sha256, verify_password},
    errors::AppError,
    models::auth::{
        AuthResponse, GroupDto, GroupWithRoleDto, LoginRequest, LogoutRequest, RefreshTokenRequest,
        RegisterRequest, TokenResponse, UserDto,
    },
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use rand::RngCore;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn local_access(State(pool): State<PgPool>) -> Result<impl IntoResponse, AppError> {
    let email = "pos@local.inventory";
    let now = chrono::Utc::now().timestamp_millis();
    let default_group_id = "default-grocery-group";

    // 1. Ensure default group exists
    sqlx::query(
        r#"
        INSERT INTO groups (group_id, group_name, description, address, created_at, last_updated, server_updated_at)
        VALUES ($1, $2, $3, $4, $5, $5, $5)
        ON CONFLICT (group_id) DO NOTHING
        "#
    )
    .bind(default_group_id)
    .bind("Local Grocery Store")
    .bind("Main grocery store inventory")
    .bind("Local Store")
    .bind(now)
    .execute(&pool)
    .await?;

    // 2. Ensure default local user exists
    let user_id = "user-local-pos";
    let default_hash = hash_password("LocalStore123").unwrap_or_default();

    sqlx::query(
        r#"
        INSERT INTO users (user_id, email, pass_hash, name, job_title, is_active, created_at, last_updated, server_updated_at)
        VALUES ($1, $2, $3, $4, $5, TRUE, $6, $6, $6)
        ON CONFLICT (user_id) DO UPDATE SET last_updated = $6, server_updated_at = $6
        "#
    )
    .bind(user_id)
    .bind(email)
    .bind(default_hash)
    .bind("Grocery Manager")
    .bind("Store Manager")
    .bind(now)
    .execute(&pool)
    .await?;

    // 3. Ensure membership
    sqlx::query(
        r#"
        INSERT INTO user_groups (user_id, group_id, role, status, joined_at, server_updated_at)
        VALUES ($1, $2, 'OWNER', 'ACTIVE', $3, $3)
        ON CONFLICT (user_id, group_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(default_group_id)
    .bind(now)
    .execute(&pool)
    .await?;

    // 4. Issue tokens
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-jwt-secret-key-32-chars-long!".to_string());
    let access_token = generate_jwt(user_id, &secret, 3600 * 24 * 30)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut random_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut random_bytes);
    let refresh_token = hex::encode(random_bytes);
    let hashed_refresh = hash_token_sha256(&refresh_token);
    let expires_at = now + (30 * 24 * 3600 * 1000);

    let refresh_id = format!("ref-{}", Uuid::new_v4());
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token, expires_at, is_revoked, created_at)
        VALUES ($1, $2, $3, $4, FALSE, $5)
        ON CONFLICT (token) DO UPDATE SET expires_at = $4, is_revoked = FALSE
        "#,
    )
    .bind(&refresh_id)
    .bind(user_id)
    .bind(&hashed_refresh)
    .bind(expires_at)
    .bind(now)
    .execute(&pool)
    .await?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        user: UserDto {
            user_id: user_id.to_string(),
            name: "Grocery Manager".to_string(),
            email: email.to_string(),
            avatar_url: None,
            phone_number: None,
            job_title: Some("Store Manager".to_string()),
            locale: None,
            time_zone: None,
            is_active: true,
            created_at: now,
            last_updated: now,
        },
        groups: vec![GroupWithRoleDto {
            group: GroupDto {
                group_id: default_group_id.to_string(),
                name: "Local Grocery Store".to_string(),
                description: Some("Main grocery store inventory".to_string()),
                address: Some("Local Store".to_string()),
                total_members: 1,
                total_products: 0,
                created_at: now,
                last_updated: now,
            },
            role: "OWNER".to_string(),
            status: "ACTIVE".to_string(),
            joined_at: now,
        }],
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Name cannot be empty".to_string()));
    }
    if !payload.email.contains('@') {
        return Err(AppError::Validation("Invalid email format".to_string()));
    }
    if payload.password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    let now = chrono::Utc::now().timestamp_millis();
    let user_id = format!("user-{}", Uuid::new_v4());
    let group_id = format!("Group-{}", Uuid::new_v4());
    let pass_hash = hash_password(&payload.password).map_err(AppError::Internal)?;

    // Create User
    sqlx::query(
        r#"
        INSERT INTO users (user_id, email, pass_hash, name, is_active, created_at, last_updated, server_updated_at)
        VALUES ($1, $2, $3, $4, TRUE, $5, $5, $5)
        "#
    )
    .bind(&user_id)
    .bind(payload.email.trim().to_lowercase())
    .bind(pass_hash)
    .bind(payload.name.trim())
    .bind(now)
    .execute(&pool)
    .await?;

    // Create Group
    let group_name = format!("{}'s Inventory", payload.name.trim());
    sqlx::query(
        r#"
        INSERT INTO groups (group_id, group_name, created_at, last_updated, server_updated_at)
        VALUES ($1, $2, $3, $3, $3)
        "#,
    )
    .bind(&group_id)
    .bind(&group_name)
    .bind(now)
    .execute(&pool)
    .await?;

    // Assign Role
    sqlx::query(
        r#"
        INSERT INTO user_groups (user_id, group_id, role, status, joined_at, server_updated_at)
        VALUES ($1, $2, 'OWNER', 'ACTIVE', $3, $3)
        "#,
    )
    .bind(&user_id)
    .bind(&group_id)
    .bind(now)
    .execute(&pool)
    .await?;

    // Tokens
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-jwt-secret-key-32-chars-long!".to_string());
    let access_token = generate_jwt(&user_id, &secret, 3600 * 24 * 30)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut random_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut random_bytes);
    let refresh_token = hex::encode(random_bytes);
    let hashed_refresh = hash_token_sha256(&refresh_token);
    let expires_at = now + (30 * 24 * 3600 * 1000);

    let reg_refresh_id = format!("ref-{}", Uuid::new_v4());
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token, expires_at, is_revoked, created_at)
        VALUES ($1, $2, $3, $4, FALSE, $5)
        "#,
    )
    .bind(&reg_refresh_id)
    .bind(&user_id)
    .bind(&hashed_refresh)
    .bind(expires_at)
    .bind(now)
    .execute(&pool)
    .await?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        user: UserDto {
            user_id: user_id.clone(),
            name: payload.name.trim().to_string(),
            email: payload.email.trim().to_lowercase(),
            avatar_url: None,
            phone_number: None,
            job_title: None,
            locale: None,
            time_zone: None,
            is_active: true,
            created_at: now,
            last_updated: now,
        },
        groups: vec![GroupWithRoleDto {
            group: GroupDto {
                group_id: group_id.clone(),
                name: group_name,
                description: None,
                address: None,
                total_members: 1,
                total_products: 0,
                created_at: now,
                last_updated: now,
            },
            role: "OWNER".to_string(),
            status: "ACTIVE".to_string(),
            joined_at: now,
        }],
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let clean_email = payload.email.trim().to_lowercase();
    let row = sqlx::query(
        "SELECT user_id, email, pass_hash, name, avatar_url, phone_number, job_title, locale, time_zone, is_active, created_at, last_updated FROM users WHERE email = $1"
    )
    .bind(&clean_email)
    .fetch_optional(&pool)
    .await?;

    let (
        user_id,
        email,
        name,
        avatar_url,
        phone_number,
        job_title,
        locale,
        time_zone,
        is_active,
        created_at,
        last_updated,
    ) = match row {
        Some(r) => {
            let pass_hash: String = r.get("pass_hash");
            if !verify_password(&payload.password, &pass_hash) {
                return Err(AppError::Auth("Invalid email or password".to_string()));
            }
            (
                r.get::<String, _>("user_id"),
                r.get::<String, _>("email"),
                r.get::<String, _>("name"),
                r.get::<Option<String>, _>("avatar_url"),
                r.get::<Option<String>, _>("phone_number"),
                r.get::<Option<String>, _>("job_title"),
                r.get::<Option<String>, _>("locale"),
                r.get::<Option<String>, _>("time_zone"),
                r.get::<bool, _>("is_active"),
                r.get::<i64, _>("created_at"),
                r.get::<i64, _>("last_updated"),
            )
        }
        None => return Err(AppError::Auth("Invalid email or password".to_string())),
    };

    if !is_active {
        return Err(AppError::Auth("Account disabled".to_string()));
    }

    let now = chrono::Utc::now().timestamp_millis();
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-jwt-secret-key-32-chars-long!".to_string());
    let access_token = generate_jwt(&user_id, &secret, 3600 * 24 * 30)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut random_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut random_bytes);
    let refresh_token = hex::encode(random_bytes);
    let hashed_refresh = hash_token_sha256(&refresh_token);
    let expires_at = now + (30 * 24 * 3600 * 1000);

    let log_refresh_id = format!("ref-{}", Uuid::new_v4());
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token, expires_at, is_revoked, created_at)
        VALUES ($1, $2, $3, $4, FALSE, $5)
        "#,
    )
    .bind(&log_refresh_id)
    .bind(&user_id)
    .bind(&hashed_refresh)
    .bind(expires_at)
    .bind(now)
    .execute(&pool)
    .await?;

    // Load user groups
    let group_rows = sqlx::query(
        r#"
        SELECT g.group_id, g.group_name, g.description, g.address, g.created_at, g.last_updated, ug.role, ug.status, ug.joined_at
        FROM user_groups ug
        JOIN groups g ON ug.group_id = g.group_id
        WHERE ug.user_id = $1
        "#
    )
    .bind(&user_id)
    .fetch_all(&pool)
    .await?;

    let groups = group_rows
        .into_iter()
        .map(|r| GroupWithRoleDto {
            group: GroupDto {
                group_id: r.get("group_id"),
                name: r.get("group_name"),
                description: r.get("description"),
                address: r.get("address"),
                total_members: 1,
                total_products: 0,
                created_at: r.get("created_at"),
                last_updated: r.get("last_updated"),
            },
            role: r.get("role"),
            status: r.get("status"),
            joined_at: r.get("joined_at"),
        })
        .collect();

    let response = AuthResponse {
        access_token,
        refresh_token,
        user: UserDto {
            user_id,
            name,
            email,
            avatar_url,
            phone_number,
            job_title,
            locale,
            time_zone,
            is_active,
            created_at,
            last_updated,
        },
        groups,
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn refresh(
    State(pool): State<PgPool>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    let hashed = hash_token_sha256(&payload.refresh_token);
    let now = chrono::Utc::now().timestamp_millis();

    let token_row =
        sqlx::query("SELECT user_id, expires_at, is_revoked FROM refresh_tokens WHERE token = $1")
            .bind(&hashed)
            .fetch_optional(&pool)
            .await?;

    let (user_id, expires_at, is_revoked) = match token_row {
        Some(r) => (
            r.get::<String, _>("user_id"),
            r.get::<i64, _>("expires_at"),
            r.get::<bool, _>("is_revoked"),
        ),
        None => {
            return Err(AppError::Auth(
                "Invalid or expired refresh token".to_string(),
            ))
        }
    };

    if is_revoked || expires_at <= now {
        return Err(AppError::Auth(
            "Invalid or expired refresh token".to_string(),
        ));
    }

    // Revoke old token
    sqlx::query("UPDATE refresh_tokens SET is_revoked = TRUE WHERE token = $1")
        .bind(&hashed)
        .execute(&pool)
        .await?;

    // Issue new tokens
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-jwt-secret-key-32-chars-long!".to_string());
    let access_token = generate_jwt(&user_id, &secret, 3600 * 24 * 30)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut random_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut random_bytes);
    let new_refresh = hex::encode(random_bytes);
    let new_hashed = hash_token_sha256(&new_refresh);
    let new_expires_at = now + (30 * 24 * 3600 * 1000);

    let new_refresh_id = format!("ref-{}", Uuid::new_v4());
    sqlx::query(
        "INSERT INTO refresh_tokens (id, user_id, token, expires_at, is_revoked, created_at) VALUES ($1, $2, $3, $4, FALSE, $5)",
    )
    .bind(&new_refresh_id)
    .bind(&user_id)
    .bind(&new_hashed)
    .bind(new_expires_at)
    .bind(now)
    .execute(&pool)
    .await?;

    Ok((
        StatusCode::OK,
        Json(TokenResponse {
            access_token,
            refresh_token: new_refresh,
        }),
    ))
}

pub async fn logout(
    State(pool): State<PgPool>,
    Json(payload): Json<LogoutRequest>,
) -> Result<impl IntoResponse, AppError> {
    let hashed = hash_token_sha256(&payload.refresh_token);
    sqlx::query("UPDATE refresh_tokens SET is_revoked = TRUE WHERE token = $1")
        .bind(&hashed)
        .execute(&pool)
        .await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"message": "Successfully logged out"})),
    ))
}
