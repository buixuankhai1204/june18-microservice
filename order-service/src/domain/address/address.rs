use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::domain;
use crate::presentation::address::address::{CreateAddressRequest, UpdateAddressRequest};

#[sea_orm::model]
#[derive(Clone, Debug, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "addresses")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    #[sea_orm(belongs_to, from = "user_id", to = "id")]
    pub user: HasOne<super::super::user::user::Entity>,
    pub title: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub country: String,
    pub city: String,
    pub postal_code: Option<String>,
    pub landmark: Option<String>,
    pub phone_number: Option<String>,
    pub status: Status,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, Deserialize, Serialize, ToSchema)]
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
    /// Business Rule: Create a new address model with validation
    pub fn create_new_address(
        request: &CreateAddressRequest,
    ) -> crate::core::error::AppResult<Self> {
        use crate::core::error::AppError;

        // Validate required fields

        if request.address_line_1.trim().is_empty() {
            return Err(AppError::BadRequestError("Address line 1 cannot be empty".to_string()));
        }

        if request.country.trim().is_empty() {
            return Err(AppError::BadRequestError("Country cannot be empty".to_string()));
        }

        if request.city.trim().is_empty() {
            return Err(AppError::BadRequestError("City cannot be empty".to_string()));
        }

        // Create and return the address model

        Ok(Self {
            id: 0, // Will be set by the database
            user_id: request.user_id,
            user: Default::default(),
            title: request.title.clone(),
            address_line_1: request.address_line_1.clone(),
            address_line_2: request.address_line_2.clone(),
            country: request.country.clone(),
            city: request.city.clone(),
            postal_code: request.postal_code.clone(),
            landmark: request.landmark.clone(),
            phone_number: request.phone_number.clone(),
            status: Status::ACTIVE,
            is_deleted: false,
            created_at: None,
            deleted_at: None,
        })
    }

    /// Business Rule: Update address model with validation
    pub fn update_from(
        mut self,
        request: &UpdateAddressRequest,
    ) -> crate::core::error::AppResult<Self> {
        use crate::core::error::AppError;


        if let Some(ref address_line_1) = request.address_line_1 {
            if address_line_1.trim().is_empty() {
                return Err(AppError::BadRequestError("Address line 1 cannot be empty".to_string()));
            }
            self.address_line_1 = address_line_1.clone();
        }

        if let Some(ref country) = request.country {
            if country.trim().is_empty() {
                return Err(AppError::BadRequestError("Country cannot be empty".to_string()));
            }
            self.country = country.clone();
        }

        if let Some(ref city) = request.city {
            if city.trim().is_empty() {
                return Err(AppError::BadRequestError("City cannot be empty".to_string()));
            }
            self.city = city.clone();
        }

        if let Some(ref title) = request.title {
            self.title = Some(title.clone());
        }

        if let Some(ref address_line_2) = request.address_line_2 {
            self.address_line_2 = Some(address_line_2.clone());
        }

        if let Some(ref postal_code) = request.postal_code {
            self.postal_code = Some(postal_code.clone());
        }

        if let Some(ref landmark) = request.landmark {
            self.landmark = Some(landmark.clone());
        }

        if let Some(ref phone_number) = request.phone_number {
            self.phone_number = Some(phone_number.clone());
        }

        Ok(self)
    }
}

