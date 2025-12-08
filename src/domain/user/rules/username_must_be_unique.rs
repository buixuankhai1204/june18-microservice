use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::infrastructure::error::{AppError, AppResult};

pub struct UsernameMustBeUnique {
    pub is_unique: bool,
}

impl BusinessRuleInterface for UsernameMustBeUnique {
    fn check_broken(&self) -> AppResult<()> {
        if !self.is_unique {
            return Err(AppError::BadRequestError(
                "Username already exists in the system".to_string(),
            ));
        }
        Ok(())
    }
}
