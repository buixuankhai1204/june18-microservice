use chrono::{NaiveDate, NaiveDateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, EnumIter};
use serde::{Deserialize, Serialize};
use crate::core::error::{AppError, AppResult};
use crate::presentation::user::user::{CreateUserRequest, UpdateUserRequest};

#[sea_orm::model]
#[derive(Clone, Debug, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub avatar: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub password: Option<String>,
    pub birth_of_date: Option<NaiveDate>,
    #[sea_orm(has_many)]
    pub address: HasMany<super::super::address::address::Entity>,
    pub phone_number: Option<String>,
    pub status: Status,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(10))")]
#[derive(PartialEq)]
pub enum Status {
    #[sea_orm(string_value = "active")]
    ACTIVE,
    #[sea_orm(string_value = "inactive")]
    INACTIVE,
}


impl ActiveModelBehavior for ActiveModel {}

// Domain Business Rules - Create and validate Models
impl ModelEx {
    /// Business Rule: Create a new user model with validation
    pub fn create_new_user(
        request: &CreateUserRequest
    ) -> crate::core::error::AppResult<Self> {
        use crate::core::error::AppError;

        // Validate required fields
        if request.first_name.trim().is_empty() {
            return Err(AppError::BadRequestError("First name cannot be empty".to_string()));
        }
        if request.last_name.trim().is_empty() {
            return Err(AppError::BadRequestError("Last name cannot be empty".to_string()));
        }
        if request.email.trim().is_empty() {
            return Err(AppError::BadRequestError("Email cannot be empty".to_string()));
        }
        
        if !request.email.contains('@') {
            return Err(AppError::BadRequestError("Email must be valid".to_string()));
        }
        
        // Create and return the user model
        Ok(Self {
            id: 0, // Will be set by the database
            avatar: request.avatar.clone(),
            first_name: request.first_name.clone(),
            last_name: request.last_name.clone(),
            username: request.username.clone(),
            email: request.email.clone(),
            password: Some(request.password.clone()), // Password will be set after hashing
            birth_of_date: request.birth_of_date,
            address: Default::default(),
            phone_number: request.phone_number.clone(),
            status: Status::ACTIVE,
            is_deleted: false,
            created_at: Some(Utc::now().naive_utc()),
            deleted_at: Some(Utc::now().naive_utc()),
        })
    }

    /// Business Rule: Update user model with validation
    pub fn update_from(
        mut self,
        request: &UpdateUserRequest
    ) -> AppResult<Self> {

        if let Some(ref first_name) = request.first_name {
            if first_name.trim().is_empty() {
                return Err(AppError::BadRequestError("First name cannot be empty".to_string()));
            }
            self.first_name = first_name.clone();
        }

        if let Some(ref last_name) = request.last_name {
            if last_name.trim().is_empty() {
                return Err(AppError::BadRequestError("Last name cannot be empty".to_string()));
            }
            self.last_name = last_name.clone();
        }

        if let Some(ref email) = request.email {
            if email.trim().is_empty() {
                return Err(AppError::BadRequestError("Email cannot be empty".to_string()));
            }
            if !email.contains('@') {
                return Err(AppError::BadRequestError("Email must be valid".to_string()));
            }
            self.email = email.clone();
        }

        if let Some(ref avatar) = request.avatar {
            self.avatar = Some(avatar.clone());
        }

        if let Some(ref birth_of_date) = request.birth_of_date {
            self.birth_of_date = Some(*birth_of_date);
        }
        if let Some(ref phone_number) = request.phone_number {
            self.phone_number = Some(phone_number.clone());
        }
        if let Some(ref status) = request.status {
            self.status = status.clone();
        }

        Ok(self)
    }
}