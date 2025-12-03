use crate::domain::address::address::{ModelEx as AddressModel, Status};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct AddressSerializer {
    pub id: i64,
    pub user_id: i64,
    pub title: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub country: String,
    pub city: String,
    pub postal_code: Option<String>,
    pub landmark: Option<String>,
    pub phone_number: Option<String>,
    pub status: Status,
    pub created_at: Option<NaiveDateTime>,
}

impl From<AddressModel> for AddressSerializer {
    fn from(value: AddressModel) -> Self {
        AddressSerializer {
            id: value.id,
            user_id: value.user_id,
            title: value.title,
            address_line_1: value.address_line_1,
            address_line_2: value.address_line_2,
            country: value.country,
            city: value.city,
            postal_code: value.postal_code,
            landmark: value.landmark,
            phone_number: value.phone_number,
            status: value.status,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct CreateAddressRequest {
    pub user_id: i64,
    pub title: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub country: String,
    pub city: String,
    pub postal_code: Option<String>,
    pub landmark: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct UpdateAddressRequest {
    pub title: Option<String>,
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub landmark: Option<String>,
    pub phone_number: Option<String>,
    pub status: Option<Status>,
}
