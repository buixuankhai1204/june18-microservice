use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::infrastructure::error::{AppError, AppResult};
use chrono::{NaiveDateTime, Utc, Duration};

pub struct VerificationResendLimitMustNotBeExceeded {
    pub resend_count: i32,
    pub last_resend_at: Option<NaiveDateTime>,
    pub max_resends_per_hour: i32,
}

impl BusinessRuleInterface for VerificationResendLimitMustNotBeExceeded {
    fn check_broken(&self) -> AppResult<()> {
        // If no previous resend, allow it
        if self.last_resend_at.is_none() {
            return Ok(());
        }

        let now = Utc::now().naive_utc();
        let one_hour_ago = now - Duration::hours(1);

        // Check if last resend was within the last hour
        if let Some(last_resend) = self.last_resend_at {
            if last_resend > one_hour_ago {
                // Within the last hour, check count
                if self.resend_count >= self.max_resends_per_hour {
                    return Err(AppError::BadRequestError(
                        format!("Maximum {} verification email resends per hour exceeded", self.max_resends_per_hour),
                    ));
                }
            }
            // If last resend was more than an hour ago, the counter should be reset (handled in service)
        }

        Ok(())
    }
}
