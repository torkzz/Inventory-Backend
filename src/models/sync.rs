use super::category::{CategoryRequest, CategoryResponse};
use super::movement::InventoryMovement;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SyncProductDto {
    pub product_id: String,
    pub category_id: Option<String>,
    pub group_id: String,
    pub name: String,
    pub brand: Option<String>,
    pub description: Option<String>,
    pub barcode: Option<String>,
    pub barcode_format: Option<String>,
    pub sku: Option<String>,
    pub image_uri: Option<String>,
    #[serde(default = "default_unit")]
    pub unit: String,
    #[serde(default)]
    pub purchase_price: f64,
    #[serde(default)]
    pub selling_price: f64,
    #[serde(default)]
    pub quantity: i32,
    #[serde(default)]
    pub minimum_quantity: i32,
    #[serde(default)]
    pub is_archived: bool,
    #[serde(default = "default_version")]
    pub version: i64,
    pub created_at: i64,
    pub last_updated: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SyncStockBatchDto {
    pub batch_id: String,
    pub product_id: String,
    pub location_id: String,
    pub group_id: String,
    pub quantity: i32,
    pub price: f64,
    pub expiration_date: Option<i64>,
    pub supplier: Option<String>,
    pub created_at: i64,
    pub last_updated: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LocationDto {
    pub location_id: String,
    pub group_id: String,
    pub name: String,
    pub description: Option<String>,
    pub location_barcode: Option<String>,
    pub created_at: i64,
    pub last_updated: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SyncPayloadDto {
    #[serde(default)]
    pub created_categories: Vec<CategoryRequest>,
    #[serde(default)]
    pub updated_categories: Vec<CategoryRequest>,
    #[serde(default)]
    pub deleted_categories: Vec<String>,

    #[serde(default)]
    pub created_locations: Vec<LocationDto>,
    #[serde(default)]
    pub updated_locations: Vec<LocationDto>,
    #[serde(default)]
    pub deleted_locations: Vec<String>,

    #[serde(default)]
    pub created_product: Vec<SyncProductDto>,
    #[serde(default)]
    pub updated_product: Vec<SyncProductDto>,
    #[serde(default)]
    pub deleted_product: Vec<String>,

    #[serde(default)]
    pub created_batches: Vec<SyncStockBatchDto>,
    #[serde(default)]
    pub updated_batches: Vec<SyncStockBatchDto>,
    #[serde(default)]
    pub deleted_batches: Vec<String>,

    #[serde(default)]
    pub movements: Vec<InventoryMovement>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SyncPullResponseDto {
    pub categories: Vec<CategoryResponse>,
    pub locations: Vec<LocationDto>,
    pub products: Vec<SyncProductDto>,
    pub batches: Vec<SyncStockBatchDto>,
    pub movements: Vec<InventoryMovement>,

    pub deleted_categories: Vec<String>,
    pub deleted_locations: Vec<String>,
    pub deleted_products: Vec<String>,
    pub deleted_batches: Vec<String>,

    pub group_id: String,
    pub server_time_stamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SyncPushResponseDto {
    pub group_id: String,
    pub success: bool,
    pub message: String,
    pub server_time_stamp: i64,
}

fn default_unit() -> String {
    "pcs".to_string()
}

fn default_version() -> i64 {
    1
}
