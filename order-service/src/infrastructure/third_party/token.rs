use crate::core::error::AppResult;
use crate::util::claim::UserClaims;
use crate::util::constant::{
    ACCESS_TOKEN_ENCODE_KEY, EXPIRE_BEARER_TOKEN_SECS, EXPIRE_REFRESH_TOKEN_SECS,
    REFRESH_TOKEN_ENCODE_KEY,
};
use uuid::Uuid;
use crate::presentation::authen::authen::TokenResponse;

pub fn service_generate_tokens(
    user_id: &i64,
    session_id: &Uuid,
) -> AppResult<TokenResponse> {
    let access_token =
        UserClaims::new(EXPIRE_BEARER_TOKEN_SECS, user_id, session_id)
            .encode(&ACCESS_TOKEN_ENCODE_KEY)?;
    let refresh_token =
        UserClaims::new(EXPIRE_REFRESH_TOKEN_SECS, user_id, session_id)
            .encode(&REFRESH_TOKEN_ENCODE_KEY)?;
    Ok(TokenResponse::new(access_token, refresh_token, EXPIRE_BEARER_TOKEN_SECS.as_secs()))
}
