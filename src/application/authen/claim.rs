use jsonwebtoken::Header;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, TokenData, Validation};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use chrono::Utc;
use utoipa::ToSchema;
use uuid::Uuid;
use crate::infrastructure::constant::{ACCESS_TOKEN_ENCODE_KEY, EXPIRE_BEARER_TOKEN_SECS, EXPIRE_REFRESH_TOKEN_SECS, REFRESH_TOKEN_ENCODE_KEY};
use crate::infrastructure::error::{AppError, AppResult};
use crate::presentation::authen::authen::TokenResponse;

pub static DECODE_HEADER: Lazy<Validation> = Lazy::new(|| Validation::new(Algorithm::RS256));
pub static ENCODE_HEADER: Lazy<Header> = Lazy::new(|| Header::new(Algorithm::RS256));

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, ToSchema)]
pub struct UserClaims {
    pub iat: i64,
    pub exp: i64,
    pub user_id: i64,
    pub sid: Uuid,
}

impl UserClaims {
    pub fn new(
        duration: Duration,
        user_id: &i64,
        session_id: &Uuid,
    ) -> Self {
        let now = Utc::now().timestamp();
        Self {
            iat: now,
            exp: now + (duration.as_secs() as i64),
            user_id: *user_id,
            sid: *session_id,
        }
    }

    pub fn decode(
        token: &str,
        key: &DecodingKey,
    ) -> Result<TokenData<Self>, jsonwebtoken::errors::Error> {
        jsonwebtoken::decode::<UserClaims>(token, key, &DECODE_HEADER)
    }

    pub fn encode(&self, key: &EncodingKey) -> Result<String, jsonwebtoken::errors::Error> {
        jsonwebtoken::encode(&ENCODE_HEADER, self, key)
    }
}

pub trait UserClaimsRequest {
    fn get_user_id(&self) -> AppResult<&i64>;
    fn get_user_claims(&self) -> AppResult<UserClaims>;
}

impl UserClaimsRequest for axum::extract::Request {
    fn get_user_id(&self) -> AppResult<&i64> {
        self.extensions()
            .get::<UserClaims>()
            .map(|u| &u.user_id)
            .ok_or_else(|| AppError::UnauthorizedError("User must login".to_string()))
    }

    fn get_user_claims(&self) -> AppResult<UserClaims> {
        self.extensions()
            .get::<UserClaims>()
            .cloned()
            .ok_or_else(|| AppError::UnauthorizedError("User must login".to_string()))
    }
}

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


pub async fn verify(password: String, hashed_pass: String) -> AppResult {
    let jh = tokio::task::spawn_blocking(move || argon_verify(password, hashed_pass));
    if let Err(err) = jh.await? {
        log::debug!("The password is not correct: {err}");
        Err(AppError::BadRequestError("The password is not correct!".to_string()))
    } else {
        Ok(())
    }
}

pub fn argon_verify(
    content: impl AsRef<str>,
    hash: impl AsRef<str>,
) -> Result<(), argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash.as_ref())?;
    Argon2::default().verify_password(content.as_ref().as_bytes(), &parsed_hash)
}
pub fn argon_hash(content: impl AsRef<str>) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon = Argon2::default();
    Ok(argon.hash_password(content.as_ref().as_bytes(), &salt)?.to_string())
}

pub async fn hash(password: String) -> AppResult<String> {
    let jh = tokio::task::spawn_blocking(move || argon_hash(password));
    let password = jh.await??;
    Ok(password)
}