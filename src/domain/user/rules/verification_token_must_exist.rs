use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::infrastructure::error::{AppError, AppResult};

pub struct VerificationTokenMustExist {
    pub token_exists: bool,
}

impl BusinessRuleInterface for VerificationTokenMustExist {
    fn check_broken(&self) -> AppResult<()> {
        if !self.token_exists {
            return Err(AppError::BadRequestError(
                "Invalid verification token".to_string(),
            ));
        }
        Ok(())
    }
}
