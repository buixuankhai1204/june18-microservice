use erp_backend::core::app_state::AppState;
use erp_backend::core::configure::app::{AppConfig, Profile};

pub mod fixtures;
pub mod helpers;

/// Create a test AppState with real configuration
/// Uses the actual database with transactions that rollback - no separate test DB needed
pub async fn setup_test_app_state() -> AppState {
    // Use local configuration for tests
    let config = AppConfig::read(Profile::Local)
        .expect("Failed to read configuration. Make sure settings/local.toml exists.");

    AppState::new(config).await.expect("Failed to create AppState")
}

/// Helper macro for running tests with automatic transaction rollback
/// Usage:
/// ```
/// test_with_rollback!(test_name, |state, tx| async move {
///     // Your test code here
///     // tx will automatically rollback when the test ends
/// });
/// ```
#[macro_export]
macro_rules! test_with_rollback {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            use sea_orm::TransactionTrait;

            let state = $crate::common::setup_test_app_state().await;
            let tx = state.db.begin().await.expect("Failed to begin transaction");

            // Run test body
            $test_body(&state, &tx).await;

            // Transaction automatically rolls back when dropped
        }
    };
}

/// Assertion helper macros
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
}

#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        match $result {
            Ok(val) => panic!("Expected Err, got Ok: {:?}", val),
            Err(e) => e,
        }
    };
}
