use thiserror::Error;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid session: {0}")]
    InvalidSession(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

pub type RedisResult<T> = Result<T, RedisError>;

impl From<serde_json::Error> for RedisError {
    fn from(err: serde_json::Error) -> Self {
        RedisError::Serialization(err.to_string())
    }
}

impl From<uuid::Error> for RedisError {
    fn from(err: uuid::Error) -> Self {
        RedisError::Parse(err.to_string())
    }
}
