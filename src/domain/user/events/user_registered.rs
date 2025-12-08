use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserRegisteredEvent {
    pub user_id: i64,
    pub email: String,
    pub full_name: String,
    pub verification_token: String,
    pub created_at: NaiveDateTime,
}

impl UserRegisteredEvent {
    pub fn new(
        user_id: i64,
        email: String,
        full_name: String,
        verification_token: String,
        created_at: NaiveDateTime,
    ) -> Self {
        Self {
            user_id,
            email,
            full_name,
            verification_token,
            created_at,
        }
    }

    pub fn topic_name() -> &'static str {
        "user_registered"
    }
}
