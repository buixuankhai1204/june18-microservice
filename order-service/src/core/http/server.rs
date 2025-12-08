use crate::api::build_routes;
use crate::core::app_state::AppState;
use crate::core::configure::app::AppConfig;
use crate::core::error::AppResult;
use axum::body::Bytes;
use axum::extract::DefaultBodyLimit;
use axum::http::{header, HeaderValue};
use fred::tracing;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::ServiceBuilderExt;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

pub struct AppServer {
    pub state: AppState,
    tcp: tokio::net::TcpListener,
}

impl AppServer {
    pub async fn new(mut config: AppConfig) -> AppResult<Self> {
        let tcp = tokio::net::TcpListener::bind(config.server.get_socket_addr()?).await?;
        let addr = tcp.local_addr()?;
        log::info!("The server is listening on: {addr}");
        config.server.port = addr.port();
        let state = AppState::new(config).await?;
        Ok(Self { state, tcp })
    }

    pub async fn run(self) -> AppResult<()> {
        let sensitive_headers: Arc<[_]> = vec![header::AUTHORIZATION, header::COOKIE].into();

        let middleware = ServiceBuilder::new()
            .sensitive_request_headers(sensitive_headers.clone())
            .layer(
                TraceLayer::new_for_http()
                    .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {})
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_response(
                        DefaultOnResponse::new()
                            .include_headers(true)
                            .latency_unit(tower_http::LatencyUnit::Millis),
                    ),
            )
            .sensitive_response_headers(sensitive_headers)
            .layer(TimeoutLayer::new(Duration::from_secs(300)))
            .compression()
            .insert_response_header_if_not_present(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/octet-stream"),
            );

        let (router, api) = OpenApiRouter::new()
            .merge(build_routes())
            .layer(DefaultBodyLimit::max(1024 * 1024 * 1000))
            .split_for_parts();

        let app = router
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
            .layer(CorsLayer::permissive())
            .layer(middleware)
            .with_state(self.state);

        axum::serve(self.tcp, app.into_make_service()).await?;
        Ok(())
    }
}
