use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::core::error::{AppError, AppResult};

pub struct UserMustNotAnEmployeeBeforeBecomeAnEmployee {
    pub is_an_employee: bool,
}

impl BusinessRuleInterface for UserMustNotAnEmployeeBeforeBecomeAnEmployee {
    fn check_broken(&self) -> AppResult<()> {
        if self.is_an_employee == false {
            Ok(())
        } else {
            Err(AppError::BadRequestError("User is already an repeat_detail_frame".to_string()))
        }
    }
}
