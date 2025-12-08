use crate::domain::user::user::{ModelEx as UserModel, Status};
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::presentation::common::SubAddressSerializer;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct UserSerializer {
    pub avatar: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub address: Vec<SubAddressSerializer>,
    pub password: Option<String>,
    pub birth_of_date: Option<NaiveDate>,
    pub phone_number: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
}

impl From<UserModel> for UserSerializer {
    fn from(value: UserModel) -> Self {
        UserSerializer {
            avatar: value.avatar,
            first_name: value.first_name,
            last_name: value.last_name,
            username: value.username,
            email: value.email,
            address: value.address.into_iter().map(|a| SubAddressSerializer {
                title: a.title,
                address_line_1: a.address_line_1,
                address_line_2: a.address_line_2,
                country: a.country,
            }).collect(),
            password: value.password,
            birth_of_date: value.birth_of_date,
            phone_number: value.phone_number,
            created_at: value.created_at,
            deleted_at: value.deleted_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct UserCreatedSerializer {
    pub user_id: String,
    pub email: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct CreateUserRequest {
    pub avatar: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub birth_of_date: Option<NaiveDate>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct UpdateUserRequest {
    pub avatar: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub birth_of_date: Option<NaiveDate>,
    pub phone_number: Option<String>,
    pub status: Option<Status>,
}
