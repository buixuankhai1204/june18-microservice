use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::infrastructure::constant::BEARER;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type")]
pub enum LoginResponse {
    Token(TokenResponse),
    Code { message: String, expire_in: u64 },
}

impl From<TokenResponse> for LoginResponse {
    fn from(value: TokenResponse) -> Self {
        LoginResponse::Token(value)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub role: String,
}

impl TokenResponse {
    pub fn new(access_token: String, refresh_token: String, expires_in: u64, user: UserInfo) -> Self {
        Self { access_token, refresh_token, expires_in, user }
    }
}
