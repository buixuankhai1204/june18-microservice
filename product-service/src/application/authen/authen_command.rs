
use crate::domain::user::user;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
#[serde(tag = "type")]
pub struct LoginByEmailCommand {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
}

impl LoginByEmailCommand {
    pub fn get_username(&self) -> &str {
        self.username.as_ref()
    }

    pub fn get_password(&self) -> &str {
        self.password.as_ref()
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate, IntoParams)]
pub struct RefreshTokenCommand {
    #[validate(length(min = 30))]
    pub token: String,
}

impl RefreshTokenCommand {
    pub fn get_token(&self) -> &str {
        self.token.as_ref()
    }
}

#[derive(Debug, Deserialize, ToSchema, Validate, IntoParams)]
pub struct ForgetPasswordCommand {
    #[validate(email)]
    pub email: String,
}

impl ForgetPasswordCommand {
    pub fn get_email(&self) -> &str {
        self.email.as_ref()
    }
}