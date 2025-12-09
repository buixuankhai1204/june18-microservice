use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use chrono::{Duration, NaiveDateTime, Utc};
use crate::infrastructure::error::{AppError, AppResult};

pub struct FailedLoginLimitMustNotBeExceeded {
    pub failed_attempts: i32,
    pub last_failed_login_at: Option<NaiveDateTime>,
    pub max_attempts: i32,
    pub lockout_window_minutes: i64,
}

impl BusinessRuleInterface for FailedLoginLimitMustNotBeExceeded {
    fn check_broken(&self) -> AppResult<()> {
        // Reset counter if more than lockout_window_minutes passed since last failed attempt
        let now = Utc::now().naive_utc();
        let mut current_attempts = self.failed_attempts;

        if let Some(last_failed) = self.last_failed_login_at {
            let window_ago = now - Duration::minutes(self.lockout_window_minutes);
            if last_failed <= window_ago {
                current_attempts = 0; // Counter reset
            }
        }

        if current_attempts >= self.max_attempts {
            return Err(AppError::AccountLockedError(
                format!("Too many failed login attempts. Account will be locked for 30 minutes.")
            ));
        }

        Ok(())
    }
}
