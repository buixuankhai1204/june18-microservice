use axum::http::{StatusCode, Uri};
use crate::core::app_state::AppState;

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
pub mod domain;

pub fn build_routes() -> OpenApiRouter<AppState> {
    let server_routes = OpenApiRouter::new()
        .routes(routes!(domain::server::health_check));
    let address_routes = OpenApiRouter::new()
        .routes(routes!(domain::address::address::controller_create_address))
        .routes(routes!(domain::address::address::controller_update_address))
        .routes(routes!(domain::address::address::controller_get_address_by_id))
        .routes(routes!(domain::address::address::controller_get_addresses_by_user_id))
        .routes(routes!(domain::address::address::controller_delete_address));
    OpenApiRouter::new()
        .merge(address_routes)
        .merge(server_routes)
        .fallback(handler_404)
}

pub async fn handler_404(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("No route for {uri}"))
}