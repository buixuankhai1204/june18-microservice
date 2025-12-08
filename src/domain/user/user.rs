use chrono::{NaiveDate, NaiveDateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, EnumIter};
use serde::{Deserialize, Serialize};
use crate::infrastructure::error::{AppError, AppResult};
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
    pub role: Role,
    pub is_deleted: bool,
    pub verification_token: Option<String>,
    pub verification_token_expiry: Option<NaiveDateTime>,
    pub email_verified_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(10))")]
#[derive(PartialEq)]
pub enum Status {
    #[sea_orm(string_value = "pending")]
    PENDING,
    #[sea_orm(string_value = "active")]
    ACTIVE,
    #[sea_orm(string_value = "inactive")]
    INACTIVE,
}

#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[derive(PartialEq)]
pub enum Role {
    #[sea_orm(string_value = "customer")]
    CUSTOMER,
    #[sea_orm(string_value = "admin")]
    ADMIN,
}


impl ActiveModelBehavior for ActiveModel {}

// Domain Business Rules - Create and validate Models
impl ModelEx {
    /// Business Rule: Create a new user for registration
    pub fn create_user_for_registration(
        email: String,
        password: String,
        full_name: String,
        phone_number: Option<String>,
        date_of_birth: Option<NaiveDate>,
    ) -> AppResult<Self> {
        // Parse full_name into first_name and last_name
        let name_parts: Vec<&str> = full_name.trim().split_whitespace().collect();
        let (first_name, last_name) = if name_parts.is_empty() {
            return Err(AppError::BadRequestError("Full name cannot be empty".to_string()));
        } else if name_parts.len() == 1 {
            (name_parts[0].to_string(), "".to_string())
        } else {
            let first = name_parts[0].to_string();
            let last = name_parts[1..].join(" ");
            (first, last)
        };

        // Generate username from email (part before @)
        let username = email.split('@').next()
            .ok_or_else(|| AppError::BadRequestError("Invalid email format".to_string()))?
            .to_string();

        // Create and return the user model
        Ok(Self {
            id: 0, // Will be set by the database
            avatar: None,
            first_name,
            last_name,
            username,
            email,
            password: Some(password), // Will be hashed before saving
            birth_of_date: date_of_birth,
            address: Default::default(),
            phone_number,
            status: Status::PENDING,
            role: Role::CUSTOMER,
            is_deleted: false,
            verification_token: None, // Will be set during registration
            verification_token_expiry: None,
            email_verified_at: None,
            created_at: Some(Utc::now().naive_utc()),
            updated_at: Some(Utc::now().naive_utc()),
            deleted_at: None,
        })
    }

    /// Business Rule: Create a new user model with validation
    pub fn create_new_user(
        request: &CreateUserRequest
    ) -> crate::infrastructure::error::AppResult<Self> {
        use crate::infrastructure::error::AppError;

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
            status: Status::PENDING,
            role: Role::CUSTOMER,
            is_deleted: false,
            verification_token: None, // Will be set during registration
            verification_token_expiry: None,
            email_verified_at: None,
            created_at: Some(Utc::now().naive_utc()),
            updated_at: Some(Utc::now().naive_utc()),
            deleted_at: None,
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