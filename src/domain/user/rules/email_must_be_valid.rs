use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::infrastructure::error::{AppError, AppResult};
use regex::Regex;

pub struct EmailMustBeValid {
    pub email: String,
}

impl BusinessRuleInterface for EmailMustBeValid {
    fn check_broken(&self) -> AppResult<()> {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .map_err(|_| AppError::BadRequestError("Invalid email regex".to_string()))?;

        if !email_regex.is_match(&self.email) {
            return Err(AppError::BadRequestError("Invalid email format".to_string()));
        }

        Ok(())
    }
}
