use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceConfig {
    pub name: String,
    pub base_url: String,
    pub health_check_path: Option<String>,
    pub timeout_secs: u64,
    pub require_auth: bool,
}

#[derive(Debug, Clone)]
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, ServiceConfig>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn with_defaults() -> Self {
        let registry = Self::new();

        // Register default services for NestJS microservices
        registry
            .register(ServiceConfig {
                name: "product-service".to_string(),
                base_url: std::env::var("PRODUCT_SERVICE_URL")
                    .unwrap_or_else(|_| "http://localhost:3002".to_string()),
                health_check_path: Some("/health".to_string()),
                timeout_secs: 30,
                require_auth: true,
            })
            .await;

        registry
            .register(ServiceConfig {
                name: "order-service".to_string(),
                base_url: std::env::var("ORDER_SERVICE_URL")
                    .unwrap_or_else(|_| "http://localhost:3003".to_string()),
                health_check_path: Some("/health".to_string()),
                timeout_secs: 30,
                require_auth: true,
            })
            .await;

        registry
            .register(ServiceConfig {
                name: "inventory-service".to_string(),
                base_url: std::env::var("INVENTORY_SERVICE_URL")
                    .unwrap_or_else(|_| "http://localhost:3004".to_string()),
                health_check_path: Some("/health".to_string()),
                timeout_secs: 30,
                require_auth: true,
            })
            .await;

        registry
            .register(ServiceConfig {
                name: "notification-service".to_string(),
                base_url: std::env::var("NOTIFICATION_SERVICE_URL")
                    .unwrap_or_else(|_| "http://localhost:3005".to_string()),
                health_check_path: Some("/health".to_string()),
                timeout_secs: 30,
                require_auth: false,
            })
            .await;

        registry
    }

    pub async fn register(&self, config: ServiceConfig) {
        let mut services = self.services.write().await;
        services.insert(config.name.clone(), config);
    }

    pub async fn get(&self, name: &str) -> Option<ServiceConfig> {
        let services = self.services.read().await;
        services.get(name).cloned()
    }

    pub async fn list_all(&self) -> Vec<ServiceConfig> {
        let services = self.services.read().await;
        services.values().cloned().collect()
    }

    pub async fn remove(&self, name: &str) -> Option<ServiceConfig> {
        let mut services = self.services.write().await;
        services.remove(name)
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
