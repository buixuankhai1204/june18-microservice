use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::core::error::{AppError, AppResult};

pub struct EmailMustBeUnique {
    pub is_unique: bool,
}

impl BusinessRuleInterface for EmailMustBeUnique {
    fn check_broken(&self) -> AppResult<()> {
        if !self.is_unique {
            return Err(AppError::BadRequestError(
                "Email already exists in the system".to_string(),
            ));
        }
        Ok(())
    }
}
