use crate::core::app_state::AppState;
use crate::core::error::{AppError, AppResult};
use crate::core::response::{
    ClientResponseError, EntityResponse, MessageResponse, ServiceStatusResponse,
};
use axum::{extract::State, Json};
use axum_extra::extract::Multipart;
use log::error;

#[utoipa::path(
    get,
    path = "/v1/server/health_check",
    tags = ["server_service"],
    responses(
        (status = 200, description = "check service is up", body = [MessageResponse])
    )
)]
pub async fn health_check() -> AppResult<Json<MessageResponse>> {
    Ok(Json(MessageResponse::new("Ok")))
}

#[utoipa::path(
    get,
    path = "/v1/server/state",
    tags = ["server_service"],
    responses(
        (status = 200, description = "state of connection services", body = [ServiceStatusResponse]),
        (status = 500, description = "internal server error", body = [ClientResponseError])
    )
)]
pub async fn server_state(State(state): State<AppState>) -> AppResult<Json<ServiceStatusResponse>> {
    let db = state.db.ping().await;
    if let Err(err) = db.as_ref() {
        error!("Database connection failed error: {err}.");
    }
    let resp = ServiceStatusResponse { db: db.is_ok(), redis: true };
    Ok(Json(resp))
}
