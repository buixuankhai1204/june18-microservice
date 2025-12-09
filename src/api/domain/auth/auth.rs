use crate::application::authen::authen_service_interface::AuthenServiceInterface;
use crate::core::app_state::AppState;
use crate::core::response::ClientResponseError;
use axum::extract::State;
use axum::Json;
use log::error;
use sea_orm::TransactionTrait;
use validator::Validate;
use crate::application::authen::authen_command::LoginByEmailCommand;
use crate::infrastructure::error::{AppError, AppResult};
use crate::presentation::authen::authen::LoginResponse;

#[utoipa::path(
    post,
    path = "/v1/login_by_email",
    request_body = LoginByEmailCommand,
    tags = ["auth_service"],
    responses(
        (status = 200, description = "Success login", body = LoginResponse),
        (status = 400, description = "Invalid data input", body = ClientResponseError),
        (status = 404, description = "Account not found", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    )
)]
pub async fn controller_login_by_email(
    State(state): State<AppState>,
    Json(cmd): Json<LoginByEmailCommand>,
) -> AppResult<Json<LoginResponse>> {
    log::info!("Login by email with request for: {}", cmd.get_email());

    // Validate command
    if let Err(validation_err) = cmd.validate() {
        return Err(AppError::BadRequestError(validation_err.to_string()));
    }

    // Begin transaction
    let tx = state.db.begin().await?;

    // Call application service
    match state
        .authen_service
        .login_by_email(&tx, &cmd)
        .await
    {
        Ok(token_response) => {
            tx.commit().await?;
            log::info!("Success login for user: {}", cmd.get_email());
            Ok(Json(LoginResponse::Token(token_response)))
        }
        Err(err) => {
            tx.rollback().await?;
            error!("Failed to login user '{}': {err:?}", cmd.get_email());
            Err(err)
        }
    }
}
