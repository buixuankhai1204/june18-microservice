use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use chrono::{NaiveDateTime, Utc};
use crate::infrastructure::error::{AppError, AppResult};

pub struct AccountMustNotBeLocked {
    pub account_locked_until: Option<NaiveDateTime>,
}

impl BusinessRuleInterface for AccountMustNotBeLocked {
    fn check_broken(&self) -> AppResult<()> {
        if let Some(locked_until) = self.account_locked_until {
            let now = Utc::now().naive_utc();
            if now < locked_until {
                let remaining_minutes = (locked_until - now).num_minutes();
                return Err(AppError::AccountLockedError(
                    format!("Account is temporarily locked due to too many failed login attempts. Please try again in {} minutes.", remaining_minutes)
                ));
            }
        }
        Ok(())
    }
}
