use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::infrastructure::error::{AppError, AppResult};
use chrono::{NaiveDate, Utc};

pub struct UserMustBeAtLeastAge {
    pub date_of_birth: Option<NaiveDate>,
    pub minimum_age: u32,
}

impl BusinessRuleInterface for UserMustBeAtLeastAge {
    fn check_broken(&self) -> AppResult<()> {
        if let Some(dob) = self.date_of_birth {
            let today = Utc::now().naive_utc().date();
            let age_years = today.years_since(dob);

            if let Some(age) = age_years {
                if age < self.minimum_age {
                    return Err(AppError::BadRequestError(
                        format!("User must be at least {} years old", self.minimum_age),
                    ));
                }
            } else {
                return Err(AppError::BadRequestError("Invalid date of birth".to_string()));
            }
        }

        Ok(())
    }
}
