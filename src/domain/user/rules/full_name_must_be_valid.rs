use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::infrastructure::error::{AppError, AppResult};

pub struct FullNameMustBeValid {
    pub full_name: String,
}

impl BusinessRuleInterface for FullNameMustBeValid {
    fn check_broken(&self) -> AppResult<()> {
        if self.full_name.trim().is_empty() {
            return Err(AppError::BadRequestError("Full name is required".to_string()));
        }

        if self.full_name.len() > 100 {
            return Err(AppError::BadRequestError(
                "Full name must not exceed 100 characters".to_string(),
            ));
        }

        Ok(())
    }
}
