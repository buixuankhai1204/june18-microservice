use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserActivatedEvent {
    pub user_id: i64,
    pub email: String,
    pub verified_at: NaiveDateTime,
}

impl UserActivatedEvent {
    pub fn new(
        user_id: i64,
        email: String,
        verified_at: NaiveDateTime,
    ) -> Self {
        Self {
            user_id,
            email,
            verified_at,
        }
    }

    pub fn topic_name() -> &'static str {
        "user_activated"
    }
}
