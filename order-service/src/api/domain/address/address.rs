use crate::core::app_state::AppState;
use crate::core::error::AppResult;
use crate::core::response::{ClientResponseError, EntityResponse};
use crate::application::address::address_service_interface::AddressServiceInterface;
use crate::presentation::address::address::{AddressSerializer, CreateAddressRequest, UpdateAddressRequest};
use axum::extract::{Path, Query, State};
use axum::Json;
use log::error;
use sea_orm::TransactionTrait;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserIdQuery {
    pub user_id: i64,
}

#[utoipa::path(
    post,
    path = "/v1/addresses",
    tags = ["address_service"],
    request_body = CreateAddressRequest,
    responses(
        (status = 201, description = "Address created successfully", body = EntityResponse<bool>),
        (status = 400, description = "Bad request", body = ClientResponseError),
        (status = 401, description = "Unauthorized", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn controller_create_address(
    State(state): State<AppState>,
    Json(request): Json<CreateAddressRequest>,
) -> AppResult<Json<EntityResponse<bool>>> {
    log::info!("Creating address for user_id: {}", request.user_id);
    let tx = state.db.begin().await?;

    match state.address_service.create_address(&tx, request).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(Json(EntityResponse {
                message: "Address created successfully.".to_string(),
                data: Some(result),
                total: 1,
            }))
        }
        Err(err) => {
            tx.rollback().await?;
            log::error!("Failed to create address: {err:?}");
            Err(err)
        }
    }
}

#[utoipa::path(
    put,
    path = "/v1/addresses/{id}",
    tags = ["address_service"],
    request_body = UpdateAddressRequest,
    params(
        ("id" = i64, Path, description = "Address ID")
    ),
    responses(
        (status = 200, description = "Address updated successfully", body = EntityResponse<bool>),
        (status = 400, description = "Bad request", body = ClientResponseError),
        (status = 401, description = "Unauthorized", body = ClientResponseError),
        (status = 404, description = "Address not found", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn controller_update_address(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateAddressRequest>,
) -> AppResult<Json<EntityResponse<bool>>> {
    log::info!("Updating address with id: {}", id);
    let tx = state.db.begin().await?;

    match state.address_service.update_address(&tx, id, request).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(Json(EntityResponse {
                message: "Address updated successfully.".to_string(),
                data: Some(result),
                total: 1,
            }))
        }
        Err(err) => {
            tx.rollback().await?;
            log::error!("Failed to update address: {err:?}");
            Err(err)
        }
    }
}

#[utoipa::path(
    get,
    path = "/v1/addresses/{id}",
    tags = ["address_service"],
    params(
        ("id" = i64, Path, description = "Address ID")
    ),
    responses(
        (status = 200, description = "Address retrieved successfully", body = EntityResponse<AddressSerializer>),
        (status = 401, description = "Unauthorized", body = ClientResponseError),
        (status = 404, description = "Address not found", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn controller_get_address_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<EntityResponse<AddressSerializer>>> {
    log::info!("Getting address with id: {}", id);
    let tx = state.db.begin().await?;

    match state.address_service.get_address_by_id(&tx, id).await {
        Ok(result) => Ok(Json(EntityResponse {
            message: "Address retrieved successfully.".to_string(),
            data: Some(result),
            total: 1,
        })),
        Err(err) => {
            log::error!("Failed to get address: {err:?}");
            Err(err)
        }
    }
}

#[utoipa::path(
    get,
    path = "/v1/addresses",
    tags = ["address_service"],
    params(
        ("user_id" = i64, Query, description = "User ID to get addresses for")
    ),
    responses(
        (status = 200, description = "Addresses retrieved successfully", body = EntityResponse<Vec<AddressSerializer>>),
        (status = 401, description = "Unauthorized", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn controller_get_addresses_by_user_id(
    State(state): State<AppState>,
    Query(params): Query<UserIdQuery>,
) -> AppResult<Json<EntityResponse<Vec<AddressSerializer>>>> {
    log::info!("Getting addresses for user_id: {}", params.user_id);
    let tx = state.db.begin().await?;

    match state.address_service.get_addresses_by_user_id(&tx, params.user_id).await {
        Ok(result) => {
            let total = result.len();
            Ok(Json(EntityResponse {
                message: "Addresses retrieved successfully.".to_string(),
                data: Some(result),
                total: total as i64,
            }))
        }
        Err(err) => {
            log::error!("Failed to get addresses: {err:?}");
            Err(err)
        }
    }
}

#[utoipa::path(
    delete,
    path = "/v1/addresses/{id}",
    tags = ["address_service"],
    params(
        ("id" = i64, Path, description = "Address ID")
    ),
    responses(
        (status = 200, description = "Address deleted successfully", body = EntityResponse<String>),
        (status = 401, description = "Unauthorized", body = ClientResponseError),
        (status = 404, description = "Address not found", body = ClientResponseError),
        (status = 500, description = "Internal server error", body = ClientResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn controller_delete_address(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<EntityResponse<String>>> {
    log::info!("Deleting address with id: {}", id);
    let tx = state.db.begin().await?;

    match state.address_service.delete_address(&tx, id).await {
        Ok(_) => {
            tx.commit().await?;
            Ok(Json(EntityResponse {
                message: "Address deleted successfully.".to_string(),
                data: Some("Address deleted successfully.".to_string()),
                total: 1,
            }))
        }
        Err(err) => {
            tx.rollback().await?;
            log::error!("Failed to delete address: {err:?}");
            Err(err)
        }
    }
}
