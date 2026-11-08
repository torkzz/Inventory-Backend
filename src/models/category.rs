use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub category_id: String,
    pub group_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub last_updated: i64,
    #[serde(default)]
    pub server_updated_at: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRequest {
    #[serde(default)]
    pub category_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub last_updated: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryResponse {
    pub category_id: String,
    pub group_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub last_updated: i64,
}

impl From<Category> for CategoryResponse {
    fn from(c: Category) -> Self {
        Self {
            category_id: c.category_id,
            group_id: c.group_id,
            name: c.name,
            description: c.description,
            created_at: c.created_at,
            last_updated: c.last_updated,
        }
    }
}
