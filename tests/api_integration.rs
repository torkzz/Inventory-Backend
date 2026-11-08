use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use inventory_backend::{create_pool, create_router, Config};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn setup_app() -> axum::Router {
    let config = Config::from_env();
    let pool = create_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");
    create_router(pool)
}

#[tokio::test]
async fn test_root_endpoint() {
    let app = setup_app().await;

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"Server running!");
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "UP");
    assert_eq!(json["database"], "connected");
}

#[tokio::test]
async fn test_auth_local() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/local")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    eprintln!("AUTH LOCAL BODY: {}", String::from_utf8_lossy(&body));
    assert_eq!(status, StatusCode::OK);
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(!json["accessToken"].as_str().unwrap().is_empty());
    assert_eq!(json["user"]["email"], "pos@local.inventory");
    assert_eq!(
        json["groups"][0]["group"]["groupId"],
        "default-grocery-group"
    );
}

#[tokio::test]
async fn test_category_and_product_crud() {
    let app = setup_app().await;

    // 1. Local auth to get token
    let auth_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/local")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(auth_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth_json: Value = serde_json::from_slice(&body).unwrap();
    let token = auth_json["accessToken"].as_str().unwrap();

    // 2. Create Category
    let cat_id = format!("cat-test-{}", uuid::Uuid::new_v4());
    let cat_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/categories")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::from(
                    json!({
                        "categoryId": cat_id,
                        "name": format!("Test Category {}", cat_id),
                        "description": "Integration test category"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(cat_res.status(), StatusCode::CREATED);

    // 3. Create Product
    let prod_id = format!("prod-test-{}", uuid::Uuid::new_v4());
    let barcode_val = format!(
        "{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(12345678)
    );
    let prod_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/products")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::from(
                    json!({
                        "productId": prod_id,
                        "categoryId": cat_id,
                        "name": "Integration Test Product",
                        "brand": "TestBrand",
                        "unit": "box",
                        "purchasePrice": 2.50,
                        "sellingPrice": 4.00,
                        "quantity": 10,
                        "minimumQuantity": 2,
                        "barcode": barcode_val
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(prod_res.status(), StatusCode::CREATED);

    // 4. Test Duplicate Barcode Rejection
    let dup_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/products")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::from(
                    json!({
                        "name": "Duplicate Barcode Product",
                        "barcode": barcode_val
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(dup_res.status(), StatusCode::CONFLICT);

    // 5. Test Stock Movement (STOCK_IN)
    let move_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/stock-in")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::from(
                    json!({
                        "productId": prod_id,
                        "quantity": 5,
                        "unit": "box",
                        "reason": "Test restock"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(move_res.status(), StatusCode::CREATED);

    // 6. Verify Product Quantity (10 + 5 = 15)
    let get_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/products/{}", prod_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_res.status(), StatusCode::OK);
    let get_body = axum::body::to_bytes(get_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let get_json: Value = serde_json::from_slice(&get_body).unwrap();
    assert_eq!(get_json["quantity"], 15);

    // 7. Clean up test product
    let del_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/products/{}", prod_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(del_res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_sync_pull_and_push() {
    let app = setup_app().await;

    let auth_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/local")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(auth_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth_json: Value = serde_json::from_slice(&body).unwrap();
    let token = auth_json["accessToken"].as_str().unwrap();

    // Pull
    let pull_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/groups/default-grocery-group/sync/pull?updatedAfter=0")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(pull_res.status(), StatusCode::OK);
    let pull_body = axum::body::to_bytes(pull_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let pull_json: Value = serde_json::from_slice(&pull_body).unwrap();
    assert!(pull_json["products"].as_array().unwrap().len() >= 40);
}
