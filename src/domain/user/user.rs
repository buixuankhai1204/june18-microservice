use chrono::{NaiveDate, NaiveDateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, EnumIter};
use serde::{Deserialize, Serialize};
use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::domain::user::rules::{UserMustNotBeAlreadyVerified, VerificationTokenMustNotBeExpired};
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
    pub verification_resend_count: i32,
    pub last_verification_resend_at: Option<NaiveDateTime>,
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
    /// Validates all business rules before creating the model
    pub fn create_user_for_registration(
        email: String,
        password: String,
        full_name: String,
        phone_number: Option<String>,
        date_of_birth: Option<NaiveDate>,
    ) -> AppResult<Self> {
        use crate::api::domain::business_rule_interface::BusinessRuleInterface;
        use crate::domain::user::rules::*;

        // Business Rule: Email must be valid
        EmailMustBeValid { email: email.clone() }.check_broken()?;

        // Business Rule: Password must meet requirements
        PasswordMustMeetRequirements { password: password.clone() }.check_broken()?;

        // Business Rule: Full name must be valid
        FullNameMustBeValid { full_name: full_name.clone() }.check_broken()?;

        // Business Rule: Phone must be valid if provided
        if let Some(ref phone) = phone_number {
            PhoneMustBeValid { phone: phone.clone() }.check_broken()?;
        }

        // Business Rule: User must be at least 13 years old
        UserMustBeAtLeastAge {
            date_of_birth,
            minimum_age: 13,
        }.check_broken()?;

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
            verification_resend_count: 0,
            last_verification_resend_at: None,
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
            verification_resend_count: 0,
            last_verification_resend_at: None,
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

    /// Business Rule: Verify user email
    /// Validates business rules and transitions user from pending to active
    pub fn verify_email(mut self) -> AppResult<Self> {

        // Business Rule: User must not be already verified
        UserMustNotBeAlreadyVerified {
            email_verified_at: self.email_verified_at,
        }.check_broken()?;

        // Business Rule: Verification token must not be expired
        VerificationTokenMustNotBeExpired {
            token_expiry: self.verification_token_expiry,
        }.check_broken()?;

        // Update user status and verification fields
        self.status = Status::ACTIVE;
        self.email_verified_at = Some(Utc::now().naive_utc());
        self.verification_token = None; // Invalidate token
        self.verification_token_expiry = None;
        self.updated_at = Some(Utc::now().naive_utc());

        Ok(self)
    }

    /// Business Rule: Prepare for verification email resend
    pub fn prepare_resend_verification(mut self, new_token: String, new_expiry: NaiveDateTime) -> AppResult<Self> {
        use crate::api::domain::business_rule_interface::BusinessRuleInterface;
        use crate::domain::user::rules::*;

        // Business Rule: User must not be already verified
        UserMustNotBeAlreadyVerified {
            email_verified_at: self.email_verified_at,
        }.check_broken()?;

        let now = Utc::now().naive_utc();

        // Reset counter if more than 1 hour has passed since last resend
        if let Some(last_resend) = self.last_verification_resend_at {
            let one_hour_ago = now - chrono::Duration::hours(1);
            if last_resend <= one_hour_ago {
                self.verification_resend_count = 0;
            }
        }

        // Business Rule: Resend limit must not be exceeded (max 3 per hour)
        VerificationResendLimitMustNotBeExceeded {
            resend_count: self.verification_resend_count,
            last_resend_at: self.last_verification_resend_at,
            max_resends_per_hour: 3,
        }.check_broken()?;

        // Update verification token and tracking fields
        self.verification_token = Some(new_token);
        self.verification_token_expiry = Some(new_expiry);
        self.verification_resend_count += 1;
        self.last_verification_resend_at = Some(now);
        self.updated_at = Some(now);

        Ok(self)
    }
}