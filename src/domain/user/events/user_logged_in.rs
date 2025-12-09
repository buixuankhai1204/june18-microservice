use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct UserLoggedInEvent {
    pub user_id: i64,
    pub email: String,
    pub session_id: String,
    pub device_info: Option<DeviceInfoEvent>,
    pub logged_in_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct DeviceInfoEvent {
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

impl UserLoggedInEvent {
    pub fn new(
        user_id: i64,
        email: String,
        session_id: String,
        device_info: Option<DeviceInfoEvent>,
        logged_in_at: NaiveDateTime,
    ) -> Self {
        Self {
            user_id,
            email,
            session_id,
            device_info,
            logged_in_at,
        }
    }

    pub fn topic_name() -> &'static str {
        "user_logged_in"
    }
}
