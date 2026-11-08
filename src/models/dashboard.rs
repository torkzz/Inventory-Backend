use super::movement::InventoryMovement;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummaryDto {
    pub total_products: i64,
    pub total_categories: i64,
    pub low_stock_count: i64,
    pub total_stock_value: f64,
    pub recent_movements: Vec<InventoryMovement>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRegistrationDto {
    pub device_id: String,
    pub device_name: String,
    pub device_model: Option<String>,
    pub group_id: String,
    pub last_sync_at: i64,
    pub registered_at: i64,
}
