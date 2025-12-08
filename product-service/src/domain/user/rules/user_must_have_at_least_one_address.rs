use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::core::error::{AppError, AppResult};

pub struct UserMustHaveAtLeastOneAddress {
    pub address_count: usize,
}

impl BusinessRuleInterface for UserMustHaveAtLeastOneAddress {
    fn check_broken(&self) -> AppResult<()> {
        if self.address_count == 0 {
            return Err(AppError::BadRequestError(
                "User must have at least one address".to_string(),
            ));
        }
        Ok(())
    }
}
