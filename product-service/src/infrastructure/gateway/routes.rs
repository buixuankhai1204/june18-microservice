use crate::core::app_state::AppState;
use crate::core::error::{AppError, AppResult};
use crate::core::response::EntityResponse;
use crate::infrastructure::gateway::proxy::{check_service_health, ProxyClient};
use crate::infrastructure::gateway::service_registry::ServiceConfig;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::Response;
use axum::response::IntoResponse;
use axum::Json;
use log::{error, info};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ServiceHealth {
    pub name: String,
    pub base_url: String,
    pub healthy: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GatewayHealth {
    pub status: String,
    pub services: Vec<ServiceHealth>,
}

/// Gateway health check
///
/// Check the health of the API gateway and all registered downstream services.
#[utoipa::path(
    get,
    path = "/gateway/health",
    tag = "Gateway",
    responses(
        (status = 200, description = "Gateway health status", body = EntityResponse<GatewayHealth>),
    )
)]
pub async fn gateway_health_check(
    State(state): State<AppState>,
) -> AppResult<Json<EntityResponse<GatewayHealth>>> {
    let services = state.gateway_registry.list_all().await;
    let client = reqwest::Client::new();

    let mut service_healths = Vec::new();
    let mut all_healthy = true;

    for service in services {
        let healthy = check_service_health(
            &client,
            &service.base_url,
            service.health_check_path.as_deref(),
        )
        .await;

        if !healthy {
            all_healthy = false;
        }

        service_healths.push(ServiceHealth {
            name: service.name.clone(),
            base_url: service.base_url.clone(),
            healthy,
        });
    }

    let status = if all_healthy { "healthy" } else { "degraded" };

    let health = GatewayHealth {
        status: status.to_string(),
        services: service_healths,
    };

    Ok(Json(EntityResponse {
        message: "Gateway health check".to_string(),
        data: Some(health),
        total: 1,
    }))
}

/// List all registered services
///
/// Retrieve a list of all services registered in the gateway.
#[utoipa::path(
    get,
    path = "/gateway/services",
    tag = "Gateway",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of services", body = EntityResponse<Vec<ServiceConfig>>),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_services(
    State(state): State<AppState>,
    claims: UserClaims,
) -> AppResult<Json<EntityResponse<Vec<ServiceConfig>>>> {
    info!("User {} listing gateway services", claims.user_id);
    let services = state.gateway_registry.list_all().await;
    let total = services.len() as i64;
    Ok(Json(EntityResponse {
        message: "Services retrieved successfully".to_string(),
        data: Some(services),
        total,
    }))
}

async fn proxy_to_service(
    service_name: &str,
    state: AppState,
    claims: Option<UserClaims>,
    request: Request,
) -> AppResult<Response<Body>> {
    // Get service configuration
    let service_config = state
        .gateway_registry
        .get(service_name)
        .await
        .ok_or_else(|| AppError::EntityNotFoundError {
            detail: format!("Service '{}' not found", service_name),
        })?;

    // Check authentication requirement
    if service_config.require_auth && claims.is_none() {
        return Err(AppError::UnauthorizedError(format!(
            "Service '{}' requires authentication",
            service_name
        )));
    }

    // Extract user context
    let (user_id, session_id) = match claims {
        Some(ref c) => (Some(c.user_id), Some(c.sid.to_string())),
        None => (None, None),
    };

    // Create proxy client
    let proxy_client = ProxyClient::new(service_config.timeout_secs)?;

    // Forward request
    proxy_client
        .forward_request(&service_config, request, user_id, session_id)
        .await
}

// Helper function to extract user claims from request
fn extract_claims_from_request(request: &Request) -> Option<UserClaims> {
    // Try to extract Authorization header
    request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth_str| {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                use crate::util::constant::ACCESS_TOKEN_DECODE_KEY;
                UserClaims::decode(token, &ACCESS_TOKEN_DECODE_KEY).ok().map(|td| td.claims)
            } else {
                None
            }
        })
}

// Proxy handlers for each service
pub async fn proxy_to_product_service(
    State(state): State<AppState>,
    request: Request,
) -> AppResult<Response<Body>> {
    let claims = extract_claims_from_request(&request);
    proxy_to_service("product-service", state, claims, request).await
}

pub async fn proxy_to_order_service(
    State(state): State<AppState>,
    request: Request,
) -> AppResult<Response<Body>> {
    let claims = extract_claims_from_request(&request);
    proxy_to_service("order-service", state, claims, request).await
}

pub async fn proxy_to_inventory_service(
    State(state): State<AppState>,
    request: Request,
) -> AppResult<Response<Body>> {
    let claims = extract_claims_from_request(&request);
    proxy_to_service("inventory-service", state, claims, request).await
}

pub async fn proxy_to_notification_service(
    State(state): State<AppState>,
    request: Request,
) -> AppResult<Response<Body>> {
    let claims = extract_claims_from_request(&request);
    proxy_to_service("notification-service", state, claims, request).await
}
