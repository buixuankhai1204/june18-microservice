use crate::application::address::address_service_interface::AddressServiceInterface;
use crate::core::error::{AppError, AppResult};
use crate::domain::address::address::Entity;
use crate::domain::address::address_repository_interface::AddressRepositoryInterface;
use crate::presentation::address::address::{AddressSerializer, CreateAddressRequest, UpdateAddressRequest};
use rdkafka::producer::FutureProducer;
use sea_orm::{DatabaseTransaction, IntoActiveModel};
use std::sync::Arc;
use utils::redis_client::RedisConnectionPool;
use crate::domain::address;

/// Application service - orchestrates domain logic, database, and external services
pub struct AddressService {
    pub redis: Arc<RedisConnectionPool>,
    pub kafka_producer: Arc<FutureProducer>,
}

impl AddressService {
    pub fn new(redis: Arc<RedisConnectionPool>, kafka_producer: Arc<FutureProducer>) -> Self {
        Self { redis, kafka_producer }
    }
}

impl AddressServiceInterface for AddressService {
    async fn create_address(
        &self,
        conn: &DatabaseTransaction,
        request: CreateAddressRequest,
    ) -> AppResult<bool> {

        // Domain: Create model with validation
        let address = address::address::ModelEx::create_new_address(
            &request
        ).map_err(
            |e| e,
        )?;

        // Infrastructure: Persist address (Model → ActiveModel in repository)
        let created_address = Entity::create_address(conn, address.into_active_model()).await?;

        // TODO: External service - Kafka event publishing
        // self.kafka_producer.send(...)

        Ok(true)
    }

    async fn update_address(
        &self,
        conn: &DatabaseTransaction,
        id: i64,
        request: UpdateAddressRequest,
    ) -> AppResult<bool> {
        // Database: Get existing address
        let existing_address = Entity::find_address_by_id(conn, id)
            .await?
            .ok_or_else(|| AppError::EntityNotFoundError {
                detail: format!("Address with id {} not found", id),
            })?;

        // Domain: Update model with validation
        let updated_model = existing_address.update_from(
            &request
        )?;

        // Infrastructure: Persist updated address (Model → ActiveModel in repository)
        let updated_address = Entity::update_address(conn, updated_model.into_active_model()).await?;

        // TODO: External service - Clear related cache if needed
        // let _ = self.redis.delete_key(...).await;

        // TODO: External service - Kafka event publishing
        // self.kafka_producer.send(...)

        Ok(true)
    }

    async fn get_address_by_id(
        &self,
        conn: &DatabaseTransaction,
        id: i64,
    ) -> AppResult<AddressSerializer> {
        // Database: Fetch address
        let address = Entity::find_address_by_id(conn, id)
            .await?
            .ok_or_else(|| AppError::EntityNotFoundError {
                detail: format!("Address with id {} not found", id),
            })?;

        Ok(AddressSerializer::from(address))
    }

    async fn delete_address(
        &self,
        conn: &DatabaseTransaction,
        id: i64,
    ) -> AppResult<bool> {
        // Database: Check if address exists
        let address = Entity::find_address_by_id(conn, id).await?;
        if address.is_none() {
            return Err(AppError::EntityNotFoundError {
                detail: format!("Address with id {} not found", id),
            });
        }

        // Database: Soft delete
        Entity::delete_address(conn, id).await?;

        // TODO: External service - Kafka event publishing
        // self.kafka_producer.send(...)

        Ok(true)
    }

    async fn get_addresses_by_user_id(
        &self,
        conn: &DatabaseTransaction,
        user_id: i64,
    ) -> AppResult<Vec<AddressSerializer>> {
        // Database: Fetch addresses for user
        let addresses = Entity::find_addresses_by_user_id(conn, user_id).await?;

        Ok(addresses.into_iter().map(|address| AddressSerializer::from(address)).collect())
    }
}
