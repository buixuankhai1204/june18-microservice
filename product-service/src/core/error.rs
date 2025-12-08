use crate::core::response::ClientResponseError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

pub type AppResult<T = ()> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Entity not found!")]
    EntityNotFoundError { detail: String },
    #[error("Entity not available!")]
    EntityNotAvailableError { detail: String },
    #[error("Entity already exists!")]
    EntityExistsError { detail: String },
    #[error("Token has expired!")]
    TokenExpiredError(String),
    #[error("{0}")]
    PermissionDeniedError(String),
    #[error("{0}")]
    UserNotActiveError(String),
    #[error("{0}")]
    InvalidSessionError(String),
    #[error("{0}")]
    ConflictError(String),
    #[error("{0}")]
    UnauthorizedError(String),
    #[error("Bad request {0}")]
    BadRequestError(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InvalidPayloadError(String),
    #[error("{0}")]
    HashError(String),
    #[error(transparent)]
    DatabaseError(#[from] sea_orm::error::DbErr),
    #[error("query_error!")]
    DatabaseErrorMessage { detail: String },
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    UuidError(#[from] uuid::Error),
    #[error(transparent)]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    RedisError(#[from] redis::RedisError),
    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),
    #[error(transparent)]
    ParseJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error(transparent)]
    AddrParseError(#[from] std::net::AddrParseError),
    #[error(transparent)]
    SpawnTaskError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    StrumParseError(#[from] strum::ParseError),
    #[error(transparent)]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error(transparent)]
    AxumError(#[from] axum::Error),
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),
    #[error(transparent)]
    TypeHeaderError(#[from] axum_extra::typed_header::TypedHeaderRejection),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, body) = self.status_and_error();
        (status_code, Json(body)).into_response()
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(value: argon2::password_hash::Error) -> Self {
        AppError::HashError(value.to_string())
    }
}

impl AppError {
    pub fn status_and_error(&self) -> (StatusCode, ClientResponseError) {
        use AppError::*;
        match self {
            InvalidPayloadError(err) => (
                StatusCode::BAD_REQUEST,
                ClientResponseError::BadRequest { detail: err.to_string() },
            ),
            BadRequestError(err) => (
                StatusCode::BAD_REQUEST,
                ClientResponseError::BadRequest { detail: err.to_string() },
            ),
            EntityNotFoundError { detail } => (
                StatusCode::BAD_REQUEST,
                ClientResponseError::EntityNotFound { detail: detail.to_string() },
            ),
            DatabaseErrorMessage { detail } => (
                StatusCode::BAD_REQUEST,
                ClientResponseError::DatabaseError { detail: detail.to_string() },
            ),
            EntityNotAvailableError { detail } => (
                StatusCode::BAD_REQUEST,
                ClientResponseError::EntityNotAvailable { detail: detail.to_string() },
            ),
            EntityExistsError { detail } => (
                StatusCode::CONFLICT,
                ClientResponseError::EntityAlreadyExists { detail: detail.to_string() },
            ),
            TokenExpiredError(_err) => {
                (StatusCode::UNAUTHORIZED, ClientResponseError::TokenExpiredError)
            },
            AxumError(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError)
            },
            ConfigError(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError)
            },
            AddrParseError(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError)
            },
            ParseJsonError(err) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ClientResponseError::UnprocessableEntity { detail: err.to_string() },
            ),
            StrumParseError(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError)
            },

            SystemTimeError(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError)
            },
            SpawnTaskError(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError)
            },
            InvalidSessionError(err) => (
                StatusCode::UNAUTHORIZED,
                ClientResponseError::BadRequest { detail: err.to_string() },
            ),
            ConflictError(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError)
            },
            UnauthorizedError(_err) => {
                (StatusCode::UNAUTHORIZED, ClientResponseError::Unauthorized)
            },
            UuidError(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError)
            },
            JwtError(_err) => (StatusCode::UNAUTHORIZED, ClientResponseError::Unauthorized),
            RedisError(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError)
            },
            HashError(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError)
            },
            ParseFloatError(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError)
            },
            DatabaseError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientResponseError::DatabaseError { detail: error.to_string() },
            ),

            Infallible(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError)
            },
            TypeHeaderError(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError)
            },
            UnknownError(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError)
            },
            _ => (StatusCode::INTERNAL_SERVER_ERROR, ClientResponseError::InternalServerError),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, utoipa::ToSchema)]
pub struct AppResponseError {
    pub kind: String,
    pub error_message: String,
    pub code: Option<i32>,
    pub details: Vec<(String, String)>,
}
