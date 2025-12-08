use crate::core::configure::app::AppConfig;
use crate::core::configure::kafka::KafkaConfig;
use crate::core::error::{AppError, AppResult};
use crate::infrastructure::persistence::postgres::{DatabaseClient, DatabaseClientExt};
use crate::application::address::address_service::AddressService;

use rdkafka::producer::FutureProducer;
use std::sync::Arc;
use utils::redis_client::RedisConnectionPool;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: Arc<DatabaseClient>,
    pub redis: Arc<RedisConnectionPool>,
    pub kafka_producer: Arc<FutureProducer>,
    pub address_service: Arc<AddressService>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> AppResult<Self> {
        let config = Arc::new(config);

        let db = Arc::new(DatabaseClient::build_from_config(&config).await?);
        let redis = Arc::new(
            RedisConnectionPool::new( &config.redis.get_url())
                .await
                .map_err(|e| AppError::BadRequestError(e.to_string()))?,
        );
        let kafka_producer = Arc::new(KafkaConfig::new().create_kafka_producer());
        let address_service =
            Arc::new(AddressService::new(redis.clone(), kafka_producer.clone()));

        Ok(Self {
            config,
            db,
            redis,
            kafka_producer,
            address_service,
        })
    }
}

impl AppState {
    pub fn producer(&self) -> &FutureProducer {
        &self.kafka_producer
    }
}
