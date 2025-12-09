use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::infrastructure::error::{AppError, AppResult};
use chrono::NaiveDateTime;

pub struct UserMustNotBeAlreadyVerified {
    pub email_verified_at: Option<NaiveDateTime>,
}

impl BusinessRuleInterface for UserMustNotBeAlreadyVerified {
    fn check_broken(&self) -> AppResult<()> {
        if self.email_verified_at.is_some() {
            return Err(AppError::BadRequestError(
                "Email is already verified".to_string(),
            ));
        }
        Ok(())
    }
}
