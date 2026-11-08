use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub user_id: String,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub phone_number: Option<String>,
    pub job_title: Option<String>,
    pub locale: Option<String>,
    pub time_zone: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
    pub last_updated: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroupDto {
    pub group_id: String,
    pub name: String,
    pub description: Option<String>,
    pub address: Option<String>,
    pub total_members: i32,
    pub total_products: i32,
    pub created_at: i64,
    pub last_updated: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroupWithRoleDto {
    pub group: GroupDto,
    pub role: String,
    pub status: String,
    pub joined_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserDto,
    pub groups: Vec<GroupWithRoleDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub confirm_password: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub aud: String,
    pub iss: String,
    pub exp: usize,
}
