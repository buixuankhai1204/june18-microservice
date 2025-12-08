use crate::core::app_state::AppState;
use crate::core::error::{AppError, AppResult};
use crate::infrastructure::error::gateway::service_registry::ServiceConfig;
use axum::body::Body;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, Request, Response};
use log::{error, info};
use reqwest::Client;
use std::time::Duration;

const HOP_BY_HOP_HEADERS: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
];

pub struct ProxyClient {
    client: Client,
}

impl ProxyClient {
    pub fn new(timeout_secs: u64) -> AppResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| AppError::BadRequestError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client })
    }

    pub async fn forward_request(
        &self,
        service_config: &ServiceConfig,
        original_request: Request<Body>,
        user_id: Option<i64>,
        session_id: Option<String>,
    ) -> AppResult<Response<Body>> {
        let method = original_request.method().clone();
        let uri = original_request.uri();

        // Extract path after service name
        let path = uri.path();
        let query = uri.query().unwrap_or("");

        // Build target URL
        let target_url = if query.is_empty() {
            format!("{}{}", service_config.base_url, path)
        } else {
            format!("{}{}?{}", service_config.base_url, path, query)
        };

        info!(
            "Proxying {} request to: {} (service: {})",
            method, target_url, service_config.name
        );

        // Build headers
        let mut headers = self.filter_headers(original_request.headers());

        // Add user context headers if authenticated
        if let Some(uid) = user_id {
            headers.insert(
                HeaderName::from_static("x-user-id"),
                HeaderValue::from_str(&uid.to_string())
                    .map_err(|e| AppError::BadRequestError(format!("Invalid user ID: {}", e)))?,
            );
        }

        if let Some(sid) = session_id {
            headers.insert(
                HeaderName::from_static("x-session-id"),
                HeaderValue::from_str(&sid)
                    .map_err(|e| AppError::BadRequestError(format!("Invalid session ID: {}", e)))?,
            );
        }

        // Get request body
        let body_bytes = axum::body::to_bytes(original_request.into_body(), usize::MAX)
            .await
            .map_err(|e| AppError::BadRequestError(format!("Failed to read request body: {}", e)))?;

        // Forward request
        let response = self
            .send_request(&method, &target_url, headers, body_bytes.to_vec())
            .await?;

        Ok(response)
    }

    async fn send_request(
        &self,
        method: &Method,
        url: &str,
        headers: HeaderMap,
        body: Vec<u8>,
    ) -> AppResult<Response<Body>> {
        let mut request_builder = match method.as_str() {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            "PATCH" => self.client.patch(url),
            "HEAD" => self.client.head(url),
            "OPTIONS" => self.client.request(reqwest::Method::OPTIONS, url),
            _ => {
                return Err(AppError::BadRequestError(format!(
                    "Unsupported HTTP method: {}",
                    method
                )))
            }
        };

        // Add headers
        for (key, value) in headers.iter() {
            if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_str().as_bytes()) {
                if let Ok(header_value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
                    request_builder = request_builder.header(header_name, header_value);
                }
            }
        }

        // Add body for non-GET requests
        if !body.is_empty() && method != Method::GET && method != Method::HEAD {
            request_builder = request_builder.body(body);
        }

        // Send request
        let response = request_builder
            .send()
            .await
            .map_err(|e| {
                error!("Failed to proxy request: {}", e);
                AppError::BadRequestError(format!("Failed to proxy request: {}", e))
            })?;

        // Convert reqwest::Response to axum::Response
        self.convert_response(response).await
    }

    async fn convert_response(&self, response: reqwest::Response) -> AppResult<Response<Body>> {
        let status = response.status();
        let headers = response.headers().clone();
        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::BadRequestError(format!("Failed to read response body: {}", e)))?;

        let mut builder = Response::builder().status(status);

        // Copy headers, filtering out hop-by-hop headers
        for (key, value) in headers.iter() {
            if !HOP_BY_HOP_HEADERS.contains(&key.as_str().to_lowercase().as_str()) {
                builder = builder.header(key, value);
            }
        }

        builder
            .body(Body::from(body_bytes))
            .map_err(|e| AppError::BadRequestError(format!("Failed to build response: {}", e)))
    }

    fn filter_headers(&self, headers: &HeaderMap) -> HeaderMap {
        let mut filtered = HeaderMap::new();
        for (key, value) in headers.iter() {
            let key_str = key.as_str().to_lowercase();
            if !HOP_BY_HOP_HEADERS.contains(&key_str.as_str()) {
                filtered.insert(key.clone(), value.clone());
            }
        }
        filtered
    }
}

pub async fn check_service_health(
    client: &Client,
    base_url: &str,
    health_path: Option<&str>,
) -> bool {
    let health_url = format!("{}{}", base_url, health_path.unwrap_or("/health"));

    match client.get(&health_url).timeout(Duration::from_secs(5)).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}
