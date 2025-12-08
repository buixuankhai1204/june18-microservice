use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::infrastructure::error::{AppError, AppResult};
use regex::Regex;

pub struct PhoneMustBeValid {
    pub phone: String,
}

impl BusinessRuleInterface for PhoneMustBeValid {
    fn check_broken(&self) -> AppResult<()> {
        let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$")
            .map_err(|_| AppError::BadRequestError("Invalid phone regex".to_string()))?;

        if !phone_regex.is_match(&self.phone) {
            return Err(AppError::BadRequestError("Invalid phone number format".to_string()));
        }

        Ok(())
    }
}
