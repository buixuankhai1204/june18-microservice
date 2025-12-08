use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct SubAddressSerializer {
    pub title: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub country: String,
}