use axum::{
    routing::{get, post, put},
    Router,
};
use sqlx::PgPool;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{
    auth, categories, dashboard, health, inventory, legacy_group, products, sync,
};

#[derive(OpenApi)]
#[openapi(
    paths(),
    components(schemas(
        crate::models::product::ProductResponse,
        crate::models::product::ProductRequest,
        crate::models::category::CategoryResponse,
        crate::models::category::CategoryRequest,
        crate::models::movement::InventoryMovement,
        crate::models::movement::CreateMovementRequest,
        crate::models::dashboard::DashboardSummaryDto,
        crate::models::auth::AuthResponse,
        crate::models::auth::TokenResponse,
    )),
    tags((name = "Inventory", description = "Grocery Inventory System API"))
)]
pub struct ApiDoc;

pub fn create_router(pool: PgPool) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Modern /api/v1 routes
    let api_v1 = Router::new()
        // Auth
        .route("/auth/local", post(auth::local_access))
        .route("/auth/login", post(auth::login))
        .route("/auth/register", post(auth::register))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        // Products
        .route(
            "/products",
            get(products::list_products).post(products::create_product),
        )
        .route(
            "/products/:id",
            get(products::get_product)
                .put(products::update_product)
                .patch(products::update_product)
                .delete(products::delete_product),
        )
        .route("/products/:id/archive", post(products::archive_product))
        .route(
            "/products/barcode/:barcode",
            get(products::get_product_by_barcode),
        )
        // Categories
        .route(
            "/categories",
            get(categories::list_categories).post(categories::create_category),
        )
        .route(
            "/categories/:id",
            get(categories::get_category)
                .put(categories::update_category)
                .delete(categories::delete_category),
        )
        // Inventory & Ledger
        .route("/inventory", get(products::list_products))
        .route(
            "/inventory/movements",
            get(inventory::list_movements).post(inventory::create_movement_generic),
        )
        .route("/inventory/movements/:id", get(inventory::get_movement))
        .route("/inventory/stock-in", post(inventory::stock_in))
        .route("/inventory/stock-out", post(inventory::stock_out))
        .route("/inventory/adjustments", post(inventory::stock_adjustment))
        .route("/inventory/low-stock", get(inventory::get_low_stock))
        .route("/inventory/expiring", get(inventory::get_expiring_stock))
        // Dashboard
        .route("/dashboard/summary", get(dashboard::get_dashboard_summary))
        .route(
            "/dashboard/category-summary",
            get(dashboard::get_category_summary),
        )
        .route("/dashboard/low-stock", get(inventory::get_low_stock))
        .route(
            "/dashboard/recent-movements",
            get(inventory::list_movements),
        )
        // Sync
        .route("/sync/push", post(sync::push_sync))
        .route("/sync/pull", get(sync::pull_sync))
        .route("/sync/register", post(sync::register_device))
        .route("/sync/devices", get(sync::list_devices));

    // Root and Legacy compatibility router
    Router::new()
        .route("/", get(health::root_handler))
        .route("/health", get(health::health_handler))
        .route("/health/ready", get(health::health_ready))
        .route("/health/live", get(health::health_live))
        .route("/system/info", get(health::system_info))
        // Legacy Auth routes
        .route("/auth/local", post(auth::local_access))
        .route("/auth/login", post(auth::login))
        .route("/auth/register", post(auth::register))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        // Legacy Group routes for Android
        .route(
            "/groups/:groupId/product",
            get(legacy_group::list_products).post(legacy_group::create_product),
        )
        .route(
            "/groups/:groupId/product/:id",
            get(legacy_group::get_product)
                .put(legacy_group::update_product)
                .delete(legacy_group::delete_product),
        )
        .route(
            "/groups/:groupId/product/:id/image",
            put(legacy_group::upload_image),
        )
        .route(
            "/groups/:groupId/category",
            get(legacy_group::list_categories).post(legacy_group::create_category),
        )
        .route(
            "/groups/:groupId/category/:id",
            get(categories::get_category)
                .put(categories::update_category)
                .delete(categories::delete_category),
        )
        .route(
            "/groups/:groupId/movements",
            get(legacy_group::list_movements).post(legacy_group::create_movement),
        )
        .route("/groups/:groupId/summary", get(legacy_group::get_summary))
        .route("/groups/:groupId/sync", post(sync::push_sync))
        .route("/groups/:groupId/sync/pull", get(sync::pull_sync))
        .route("/sync/ws/:groupId", get(sync::sync_ws_handler))
        // API v1 namespace
        .nest("/api/v1", api_v1)
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Uploads static directory
        .nest_service("/uploads", ServeDir::new("uploads"))
        .layer(cors)
        .with_state(pool)
}
