use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InventoryMovement {
    pub movement_id: String,
    pub product_id: String,
    pub group_id: String,
    pub movement_type: String, // STOCK_IN, STOCK_OUT, ADJUSTMENT, INITIAL_BALANCE
    pub quantity: i32,
    pub unit: String,
    pub reason: Option<String>,
    pub source: String, // ANDROID, WEB, SYSTEM
    pub device_id: Option<String>,
    pub created_by: Option<String>,
    pub created_at: i64,
    pub server_updated_at: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateMovementRequest {
    #[serde(default)]
    pub movement_id: Option<String>,
    pub product_id: String,
    #[serde(default)]
    pub movement_type: String,
    pub quantity: i32,
    #[serde(default = "default_unit")]
    pub unit: String,
    pub reason: Option<String>,
    #[serde(default = "default_source")]
    pub source: String,
    pub device_id: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<i64>,
}

fn default_unit() -> String {
    "pcs".to_string()
}

fn default_source() -> String {
    "WEB".to_string()
}
