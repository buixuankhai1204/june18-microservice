use crate::core::app_state::AppState;
use crate::core::response::{ClientResponseError, EntityResponse};
use crate::application::user::user_service_interface::UserServiceInterface;
use crate::application::user::user_command::{RegisterUserCommand, VerifyEmailCommand, ResendVerificationEmailCommand};
use crate::presentation::user::user::{UserSerializer, CreateUserRequest, UpdateUserRequest, UserCreatedSerializer};
use axum::extract::{Path, Query, State};
use axum::Json;
use axum::http::StatusCode;
use log::error;
use sea_orm::TransactionTrait;
use serde::Deserialize;
use crate::infrastructure::error::AppResult;
use crate::application::authen::claim::UserClaims;

#[utoipa::path(
    get,
    path = "/v1/me",
    tags = ["user_service"],
    responses(
        (status = 200, description = "Success get user profile", body =
        EntityResponse<UserSerializer>),
        (status = 401, description = "Unauthorized", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn controller_get_profile(
    State(state): State<AppState>,
    claims: UserClaims,
) -> AppResult<Json<EntityResponse<UserSerializer>>> {
    log::info!("Get profile user id: {}.", claims.user_id);
    let tx = state.db.begin().await?;
    match state.user_service.get_profile(&tx, claims.user_id).await {
        Ok(result) => Ok(Json(EntityResponse {
            message: "Successfully get profile.".to_string(),
            data: Some(result),
            total: 1,
        })),
        Err(err) => {
            log::warn!("Unsuccessfully get profile user: {err:?}.");
            Err(err)
        },
    }
}

#[utoipa::path(
    post,
    path = "/v1/logout",
    tags = ["user_service"],
    responses(
        (status = 200, description = "Success logout", body = EntityResponse<String>),
        (status = 401, description = "Unauthorized", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn controller_logout(
    State(state): State<AppState>,
    claims: UserClaims,
) -> AppResult<Json<EntityResponse<String>>> {
    log::info!("Logout user id: {}", claims.user_id);
    let tx = state.db.begin().await?;

    match state.user_service.logout(&tx, claims.user_id).await {
        Ok(_) => {
            log::info!("Success logout user id: {}", claims.user_id);
            Ok(Json(EntityResponse {
                message: "Successfully logged out.".to_string(),
                data: Some("Successfully logged out.".to_string()),
                total: 1,
            }))
        },
        Err(err) => {
            error!("Unsuccessfully logout user: {err:?}");
            Err(err)
        },
    }
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

fn default_page() -> u64 {
    0
}

fn default_page_size() -> u64 {
    10
}

#[utoipa::path(
    post,
    path = "/v1/auth/register",
    tags = ["user_service"],
    request_body = RegisterUserCommand,
    responses(
        (status = 201, description = "User registered successfully", body = EntityResponse<UserCreatedSerializer>),
        (status = 400, description = "Bad request - validation failed", body = ClientResponseError),
        (status = 409, description = "Conflict - email or phone already exists", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    )
)]
pub async fn controller_register_user(
    State(state): State<AppState>,
    Json(command): Json<RegisterUserCommand>,
) -> Result<(StatusCode, Json<EntityResponse<UserCreatedSerializer>>), crate::infrastructure::error::AppError> {
    log::info!("Registering user with email: {}", command.email);
    let tx = state.db.begin().await?;

    match state.user_service.register_user(&tx, command).await {
        Ok(result) => {
            tx.commit().await?;
            log::info!("User registered successfully: {}", result.user_id);
            Ok((
                StatusCode::CREATED,
                Json(EntityResponse {
                    message: "User registered successfully.".to_string(),
                    data: Some(result),
                    total: 1,
                }),
            ))
        }
        Err(err) => {
            tx.rollback().await?;
            log::error!("Failed to register user: {err:?}");
            Err(err)
        }
    }
}

#[utoipa::path(
    post,
    path = "/v1/auth/verify-email",
    tags = ["user_service"],
    request_body = VerifyEmailCommand,
    responses(
        (status = 200, description = "Email verified successfully", body = EntityResponse<bool>),
        (status = 400, description = "Bad request - invalid or expired token", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    )
)]
pub async fn controller_verify_email(
    State(state): State<AppState>,
    Json(command): Json<VerifyEmailCommand>,
) -> Result<(StatusCode, Json<EntityResponse<bool>>), crate::infrastructure::error::AppError> {
    log::info!("Verifying email with token: {}", command.verification_token);
    let tx = state.db.begin().await?;

    match state.user_service.verify_email(&tx, command).await {
        Ok(result) => {
            tx.commit().await?;
            log::info!("Email verified successfully");
            Ok((
                StatusCode::OK,
                Json(EntityResponse {
                    message: "Email verified successfully. You can now login.".to_string(),
                    data: Some(result),
                    total: 1,
                }),
            ))
        }
        Err(err) => {
            tx.rollback().await?;
            log::error!("Failed to verify email: {err:?}");
            Err(err)
        }
    }
}

#[utoipa::path(
    post,
    path = "/v1/auth/resend-verification",
    tags = ["user_service"],
    request_body = ResendVerificationEmailCommand,
    responses(
        (status = 200, description = "Verification email resent successfully", body = EntityResponse<bool>),
        (status = 400, description = "Bad request - email already verified or rate limit exceeded", body = ClientResponseError),
        (status = 404, description = "User not found", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    )
)]
pub async fn controller_resend_verification_email(
    State(state): State<AppState>,
    Json(command): Json<ResendVerificationEmailCommand>,
) -> Result<(StatusCode, Json<EntityResponse<bool>>), crate::infrastructure::error::AppError> {
    log::info!("Resending verification email for: {}", command.email);
    let tx = state.db.begin().await?;

    match state.user_service.resend_verification_email(&tx, command).await {
        Ok(result) => {
            tx.commit().await?;
            log::info!("Verification email resent successfully");
            Ok((
                StatusCode::OK,
                Json(EntityResponse {
                    message: "Verification email has been resent. Please check your inbox.".to_string(),
                    data: Some(result),
                    total: 1,
                }),
            ))
        }
        Err(err) => {
            tx.rollback().await?;
            log::error!("Failed to resend verification email: {err:?}");
            Err(err)
        }
    }
}

#[utoipa::path(
    post,
    path = "/v1/users",
    tags = ["user_service"],
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = EntityResponse<bool>),
        (status = 400, description = "Bad request", body = ClientResponseError),
        (status = 409, description = "User already exists", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    )
)]
pub async fn controller_create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> AppResult<Json<EntityResponse<bool>>> {
    log::info!("Creating user with username: {}", request.username);
    let tx = state.db.begin().await?;

    match state.user_service.create_user(&tx, request).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(Json(EntityResponse {
                message: "User created successfully.".to_string(),
                data: Some(result),
                total: 1,
            }))
        }
        Err(err) => {
            tx.rollback().await?;
            log::error!("Failed to create user: {err:?}");
            Err(err)
        }
    }
}

#[utoipa::path(
    put,
    path = "/v1/users/{id}",
    tags = ["user_service"],
    request_body = UpdateUserRequest,
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User updated successfully", body = EntityResponse<bool>),
        (status = 400, description = "Bad request", body = ClientResponseError),
        (status = 401, description = "Unauthorized", body = ClientResponseError),
        (status = 404, description = "User not found", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn controller_update_user(
    State(state): State<AppState>,
    _claims: UserClaims,
    Path(id): Path<i64>,
    Json(request): Json<UpdateUserRequest>,
) -> AppResult<Json<EntityResponse<bool>>> {
    log::info!("Updating user with id: {}", id);
    let tx = state.db.begin().await?;

    match state.user_service.update_user(&tx, id, request).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(Json(EntityResponse {
                message: "User updated successfully.".to_string(),
                data: Some(result),
                total: 1,
            }))
        }
        Err(err) => {
            tx.rollback().await?;
            log::error!("Failed to update user: {err:?}");
            Err(err)
        }
    }
}

#[utoipa::path(
    get,
    path = "/v1/users/{id}",
    tags = ["user_service"],
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User retrieved successfully", body = EntityResponse<UserSerializer>),
        (status = 401, description = "Unauthorized", body = ClientResponseError),
        (status = 404, description = "User not found", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn controller_get_user_by_id(
    State(state): State<AppState>,
    _claims: UserClaims,
    Path(id): Path<i64>,
) -> AppResult<Json<EntityResponse<UserSerializer>>> {
    log::info!("Getting user with id: {}", id);
    let tx = state.db.begin().await?;

    match state.user_service.get_profile(&tx, id).await {
        Ok(result) => Ok(Json(EntityResponse {
            message: "User retrieved successfully.".to_string(),
            data: Some(result),
            total: 1,
        })),
        Err(err) => {
            log::error!("Failed to get user: {err:?}");
            Err(err)
        }
    }
}

#[utoipa::path(
    get,
    path = "/v1/users",
    tags = ["user_service"],
    params(
        ("page" = Option<u64>, Query, description = "Page number (default: 0)"),
        ("page_size" = Option<u64>, Query, description = "Page size (default: 10)")
    ),
    responses(
        (status = 200, description = "Users retrieved successfully", body = EntityResponse<Vec<UserSerializer>>),
        (status = 401, description = "Unauthorized", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn controller_list_users(
    State(state): State<AppState>,
    _claims: UserClaims,
    Query(params): Query<PaginationQuery>,
) -> AppResult<Json<EntityResponse<Vec<UserSerializer>>>> {
    log::info!("Listing users - page: {}, page_size: {}", params.page, params.page_size);
    let tx = state.db.begin().await?;

    match state.user_service.list_users(&tx, params.page, params.page_size).await {
        Ok(result) => {
            let total = result.len();
            Ok(Json(EntityResponse {
                message: "Users retrieved successfully.".to_string(),
                data: Some(result),
                total: total as i64,
            }))
        }
        Err(err) => {
            log::error!("Failed to list users: {err:?}");
            Err(err)
        }
    }
}

#[utoipa::path(
    delete,
    path = "/v1/users/{id}",
    tags = ["user_service"],
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User deleted successfully", body = EntityResponse<String>),
        (status = 401, description = "Unauthorized", body = ClientResponseError),
        (status = 404, description = "User not found", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn controller_delete_user(
    State(state): State<AppState>,
    _claims: UserClaims,
    Path(id): Path<i64>,
) -> AppResult<Json<EntityResponse<String>>> {
    log::info!("Deleting user with id: {}", id);
    let tx = state.db.begin().await?;

    match state.user_service.delete_user(&tx, id).await {
        Ok(_) => {
            tx.commit().await?;
            Ok(Json(EntityResponse {
                message: "User deleted successfully.".to_string(),
                data: Some("User deleted successfully.".to_string()),
                total: 1,
            }))
        }
        Err(err) => {
            tx.rollback().await?;
            log::error!("Failed to delete user: {err:?}");
            Err(err)
        }
    }
}
