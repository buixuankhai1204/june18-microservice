use crate::core::configure::app::AppConfig;
use crate::core::configure::kafka::KafkaConfig;
use crate::infrastructure::persistence::postgres::{DatabaseClient, DatabaseClientExt};
use crate::infrastructure::persistence::redis_client::RedisConnectionPool;
use crate::application::user::user_service::UserService;
use crate::application::authen::authen_service::AuthenService;
use crate::application::address::address_service::AddressService;
use crate::infrastructure::gateway::service_registry::ServiceRegistry;

use rdkafka::producer::FutureProducer;
use std::sync::Arc;
use crate::infrastructure::error::{AppError, AppResult};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: Arc<DatabaseClient>,
    pub redis: Arc<RedisConnectionPool>,
    pub kafka_producer: Arc<FutureProducer>,
    pub user_service: Arc<UserService>,
    pub authen_service: Arc<AuthenService>,
    pub address_service: Arc<AddressService>,
    pub gateway_registry: Arc<ServiceRegistry>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> AppResult<Self> {
        let config = Arc::new(config);

        let db = Arc::new(DatabaseClient::build_from_config(&config).await?);
        let redis = Arc::new(
            RedisConnectionPool::new(&config.redis.get_url())
                .await
                .map_err(|e| AppError::BadRequestError(e.to_string()))?
        );
        let kafka_producer = Arc::new(KafkaConfig::new().create_kafka_producer());
        let authen_service =
            Arc::new(AuthenService::new(redis.clone(), kafka_producer.clone()));
        let user_service =
            Arc::new(UserService::new(redis.clone(), kafka_producer.clone()));
        let address_service =
            Arc::new(AddressService::new(redis.clone(), kafka_producer.clone()));
        let gateway_registry = Arc::new(ServiceRegistry::with_defaults().await);

        Ok(Self {
            config,
            db,
            redis,
            authen_service,
            kafka_producer,
            user_service,
            address_service,
            gateway_registry,
        })
    }
}

impl AppState {
    pub fn producer(&self) -> &FutureProducer {
        &self.kafka_producer
    }
}
