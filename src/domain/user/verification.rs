use chrono::{Duration, NaiveDateTime, Utc};
use uuid::Uuid;

/// Generate a verification token with expiry
pub fn generate_verification_token() -> (String, NaiveDateTime) {
    let token = Uuid::new_v4().to_string();
    let expiry = Utc::now().naive_utc() + Duration::hours(24);
    (token, expiry)
}

/// Check if verification token has expired
pub fn is_token_expired(expiry: &NaiveDateTime) -> bool {
    let now = Utc::now().naive_utc();
    now > *expiry
}
