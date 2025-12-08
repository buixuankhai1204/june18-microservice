use crate::core::error::{AppError, AppResult};
use crate::infrastructure::third_party::redis::lib::RedisConnectionPool;
use crate::util::claim::UserClaims;
use std::str::FromStr;
use uuid::Uuid;

pub async fn is_valid_session(
    redis: &RedisConnectionPool,
    claims: &UserClaims,
    is_del_session: bool,
) -> AppResult<i64> {
    let session_key = claims.user_id.to_string();
    let session_id: Option<String> =
        redis.get_key::<Option<String>>(&session_key.clone().into()).await.unwrap();
    if claims.sid != Uuid::from_str(session_id.unwrap().as_str())? {
        if is_del_session {
            redis.delete_key(&session_key.into()).await.unwrap();
        }
        return Err(AppError::InvalidSessionError("Session is invalid".to_string()));
    }
    Ok(claims.user_id)
}

pub fn generate(user_uuid: &Uuid) -> (String, Uuid) {
    let session_id = Uuid::new_v4();
    (user_uuid.to_string(), session_id)
}
