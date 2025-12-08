use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::infrastructure::error::{AppError, AppResult};

pub struct PasswordMustMeetRequirements {
    pub password: String,
}

impl BusinessRuleInterface for PasswordMustMeetRequirements {
    fn check_broken(&self) -> AppResult<()> {
        // Min 8 chars
        if self.password.len() < 8 {
            return Err(AppError::BadRequestError(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        // Must contain uppercase
        let has_uppercase = self.password.chars().any(|c| c.is_uppercase());
        if !has_uppercase {
            return Err(AppError::BadRequestError(
                "Password must contain at least one uppercase letter".to_string(),
            ));
        }

        // Must contain lowercase
        let has_lowercase = self.password.chars().any(|c| c.is_lowercase());
        if !has_lowercase {
            return Err(AppError::BadRequestError(
                "Password must contain at least one lowercase letter".to_string(),
            ));
        }

        // Must contain digit
        let has_digit = self.password.chars().any(|c| c.is_numeric());
        if !has_digit {
            return Err(AppError::BadRequestError(
                "Password must contain at least one number".to_string(),
            ));
        }

        // Must contain special character
        let has_special = self.password.chars().any(|c| !c.is_alphanumeric());
        if !has_special {
            return Err(AppError::BadRequestError(
                "Password must contain at least one special character".to_string(),
            ));
        }

        Ok(())
    }
}
