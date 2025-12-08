use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TestClaims {
    pub user_id: i64,
    pub department_id: Option<i64>,
    pub exp: usize,
}

/// Generate a test JWT token
pub fn generate_test_token(user_id: i64, department_id: Option<i64>) -> String {
    let claims = TestClaims {
        user_id,
        department_id,
        exp: 10000000000, // Far future expiration
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(
            "test_secret_key_for_testing_only_do_not_use_in_production".as_ref(),
        ),
    )
    .expect("Failed to generate test token")
}

/// Create test user claims
pub fn create_test_user_claims(
    user_id: i64,
    department_id: Option<i64>,
) -> erp_backend::util::claim::UserClaims {
    erp_backend::util::claim::UserClaims {
        user_id,
        department_id,
        exp: 10000000000,
        iat: chrono::Utc::now().timestamp(),
        sid: uuid::Uuid::new_v4(),
    }
}

/// Helper to check if a result contains a specific error message
pub fn assert_error_contains(result: &erp_backend::core::error::AppError, expected: &str) -> bool {
    match result {
        erp_backend::core::error::AppError::BadRequestError(msg) => msg.contains(expected),
        erp_backend::core::error::AppError::NotFound(msg) => msg.contains(expected),
        erp_backend::core::error::AppError::UnauthorizedError(msg) => msg.contains(expected),
        erp_backend::core::error::AppError::EntityNotFoundError { detail } => {
            detail.contains(expected)
        },
        erp_backend::core::error::AppError::EntityNotAvailableError { detail } => {
            detail.contains(expected)
        },
        erp_backend::core::error::AppError::InvalidPayloadError(msg) => msg.contains(expected),
        _ => false,
    }
}

/// Helper to wait for async operations
pub async fn wait_for_condition<F, Fut>(mut check: F, max_attempts: u32) -> bool
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    for _ in 0..max_attempts {
        if check().await {
            return true;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    false
}
