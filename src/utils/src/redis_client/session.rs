use super::error::{RedisError, RedisResult};
use super::instance::RedisConnectionPool;
use std::str::FromStr;
use uuid::Uuid;

/// Validate a session by checking if the session ID in Redis matches the expected session ID
pub async fn is_valid_session(
    redis: &RedisConnectionPool,
    user_id: i64,
    expected_session_id: &Uuid,
    delete_on_invalid: bool,
) -> RedisResult<i64> {
    let session_key = user_id.to_string();
    let session_id: Option<String> = redis.get_key::<String>(&session_key).await?;

    match session_id {
        Some(sid) => {
            let stored_session_id = Uuid::from_str(&sid)?;
            if expected_session_id != &stored_session_id {
                if delete_on_invalid {
                    redis.delete_key(&session_key).await?;
                }
                return Err(RedisError::InvalidSession("Session is invalid".to_string()));
            }
            Ok(user_id)
        }
        None => Err(RedisError::NotFound(format!("Session not found for user {}", user_id))),
    }
}

/// Generate a new session ID
pub fn generate_session_id(user_id: &i64) -> (String, Uuid) {
    let session_id = Uuid::new_v4();
    (user_id.to_string(), session_id)
}
