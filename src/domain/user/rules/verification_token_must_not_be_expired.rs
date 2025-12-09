use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::infrastructure::error::{AppError, AppResult};
use chrono::NaiveDateTime;

pub struct VerificationTokenMustNotBeExpired {
    pub token_expiry: Option<NaiveDateTime>,
}

impl BusinessRuleInterface for VerificationTokenMustNotBeExpired {
    fn check_broken(&self) -> AppResult<()> {
        use crate::domain::user::verification::is_token_expired;

        if let Some(expiry) = self.token_expiry {
            if is_token_expired(&expiry) {
                return Err(AppError::BadRequestError(
                    "Verification token has expired".to_string(),
                ));
            }
        } else {
            return Err(AppError::BadRequestError(
                "Verification token expiry not found".to_string(),
            ));
        }

        Ok(())
    }
}
