use crate::core::app_state::AppState;
use crate::core::error::AppError;
use crate::util::claim::UserClaims;
use crate::util::constant::ACCESS_TOKEN_DECODE_KEY;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::RequestPartsExt;
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use log::error;

impl FromRequestParts<AppState> for UserClaims {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
            Ok(header) => {
                let TypedHeader(Authorization(bearer)) = header;
                let user_claims =
                    UserClaims::decode(bearer.token(), &ACCESS_TOKEN_DECODE_KEY)?.claims;
                // redis_client::session::is_valid_session(&state.redis, &user_claims, false).await?;
                Ok(user_claims)
            },
            Err(err) => {
                error!("{}", err);
                Err(AppError::UnauthorizedError(err.to_string()))?
            },
        }
    }
}
