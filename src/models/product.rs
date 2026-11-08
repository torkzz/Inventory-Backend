use super::category::CategoryResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProductRow {
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
    pub unit: String,
    pub purchase_price: f64,
    pub selling_price: f64,
    pub quantity: i32,
    pub minimum_quantity: i32,
    pub is_archived: bool,
    pub version: i64,
    pub created_at: i64,
    pub last_updated: i64,
    pub server_updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductResponse {
    pub product_id: String,
    pub group_id: String,
    pub category: Option<CategoryResponse>,
    pub name: String,
    pub brand: Option<String>,
    pub description: Option<String>,
    pub barcode: Option<String>,
    pub barcode_format: Option<String>,
    pub sku: Option<String>,
    pub image_uri: Option<String>,
    pub unit: String,
    pub purchase_price: f64,
    pub selling_price: f64,
    pub quantity: i32,
    pub minimum_quantity: i32,
    pub is_archived: bool,
    pub version: i64,
    pub batches: Vec<serde_json::Value>,
    pub created_at: i64,
    pub last_updated: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductRequest {
    pub product_id: Option<String>,
    pub category_id: Option<String>,
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
    #[serde(default)]
    pub batches: Vec<serde_json::Value>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub last_updated: Option<i64>,
}

fn default_unit() -> String {
    "pcs".to_string()
}

fn default_version() -> i64 {
    1
}
