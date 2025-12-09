use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::domain::user::user::Status;
use crate::infrastructure::error::{AppError, AppResult};

pub struct AccountMustBeActive {
    pub status: Status,
}

impl BusinessRuleInterface for AccountMustBeActive {
    fn check_broken(&self) -> AppResult<()> {
        if self.status != Status::ACTIVE {
            return Err(AppError::UnauthorizedError(
                "Account is not active. Please verify your email or contact support.".to_string()
            ));
        }
        Ok(())
    }
}
