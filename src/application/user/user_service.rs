use crate::core::error::{AppError, AppResult};
use crate::infrastructure::third_party::redis::lib::RedisConnectionPool;
use crate::application::user::user_service_interface::UserServiceInterface;
use crate::domain::user::user_repository_interface::UserRepositoryInterface;
use crate::presentation::user::user::{UserSerializer, CreateUserRequest, UpdateUserRequest};
use crate::util::password;
use log::error;
use rdkafka::producer::FutureProducer;
use sea_orm::{DatabaseTransaction, IntoActiveModel};
use std::sync::Arc;
use crate::domain::user;

/// Application service - orchestrates domain logic, database, and external services
#[derive()]
pub struct UserService {
    pub redis: Arc<RedisConnectionPool>,
    pub kafka_producer: Arc<FutureProducer>,
}

impl UserService {
    pub fn new(redis: Arc<RedisConnectionPool>, kafka_producer: Arc<FutureProducer>) -> Self {
        Self { redis, kafka_producer }
    }
}

impl UserServiceInterface for UserService {
    async fn create_user(
        &self,
        conn: &DatabaseTransaction,
        request: CreateUserRequest,
    ) -> AppResult<bool> {
        // Database: Check username uniqueness
        if user::user::Entity::username_exists(conn, &request.username).await? {
            return Err(AppError::EntityExistsError {
                detail: format!("Username {} already exists", request.username),
            });
        }

        // Database: Check email uniqueness
        if user::user::Entity::email_exists(conn, &request.email).await? {
            return Err(AppError::EntityExistsError {
                detail: format!("Email {} already exists", request.email),
            });
        }

        // External service: Hash password
        let hashed_password = password::hash(request.password.clone()).await?;

        let user = user::user::ModelEx::create_new_user(
            &request
        )?;



        // Infrastructure: Persist user (Model → ActiveModel in repository)
        let created_user = user::user::Entity::create_user(conn, user.into_active_model()).await?;


        // TODO: External service - Kafka event publishing
        // self.kafka_producer.send(...)

        Ok(true)
    }

    async fn update_user(
        &self,
        conn: &DatabaseTransaction,
        id: i64,
        request: UpdateUserRequest,
    ) -> AppResult<bool> {
        // Database: Get existing user
        let existing_user_opt = user::user::Entity::find_user_by_id(conn, id).await?;
        let existing_user = existing_user_opt.ok_or_else(|| AppError::EntityNotFoundError {
            detail: format!("User with id {} not found", id),
        })?;

        // Database: Check email uniqueness if changing
        if let Some(ref email) = request.email {
            if email != &existing_user.email {
                if user::user::Entity::email_exists(conn, email).await? {
                    return Err(AppError::EntityExistsError {
                        detail: format!("Email {} already exists", email),
                    });
                }
            }
        }

        // Convert ModelEx to Model (remove relationships for update)

        // Domain: Update model with validation
        let updated_model = existing_user.update_from(
            &request
        )?;

        // Infrastructure: Persist updated user (Model → ActiveModel in repository)
        let updated_user = user::user::Entity::update_user(conn, updated_model.into_active_model()).await?;

        // External service: Clear Redis cache
        let _ = self.redis.delete_key(&format!("profile:user_id:{}", id).to_string().into()).await;

        // TODO: External service - Kafka event publishing
        // self.kafka_producer.send(...)

        Ok(true)
    }

    async fn get_profile(
        &self,
        conn: &DatabaseTransaction,
        user_id: i64,
    ) -> AppResult<UserSerializer> {
        // External service: Try Redis cache first
        let info_user = self
            .redis
            .get_and_deserialize_key::<UserSerializer>(
                &format!("profile:user_id:{}", user_id).to_string().into(),
                "UserRelatedResponse",
            )
            .await;

        match info_user {
            Ok(value) => Ok(value),
            Err(error) => {
                error!("Error when get profile from redis: {:#?}", error);

                // Database: Fetch from database
                match user::user::Entity::find_user_by_id(conn, user_id).await {
                    Ok(Some(profile)) => {
                        // External service: Cache in Redis
                        let _ = self
                            .redis
                            .serialize_and_set_key_with_expiry(
                                &format!("profile:user_id:{}", user_id.to_string())
                                    .to_string()
                                    .into(),
                                &profile,
                                88640,
                            )
                            .await;
                        Ok(UserSerializer::from(profile))
                    },
                    Err(_error) => Err(AppError::EntityNotFoundError {
                        detail: format!("User not found by id {}", user_id),
                    }),
                    _ => {
                        Err(AppError::EntityNotFoundError {
                            detail: format!("User not found by id {}", user_id),
                        })
                    }
                }
            },
        }
    }

    async fn delete_user(
        &self,
        conn: &DatabaseTransaction,
        id: i64,
    ) -> AppResult<bool> {
        // Database: Check if user exists
        let user = user::user::Entity::find_user_by_id(conn, id).await?;
        if user.is_none() {
            return Err(AppError::EntityNotFoundError {
                detail: format!("User with id {} not found", id),
            });
        }

        // Database: Soft delete
        user::user::Entity::delete_user(conn, id).await?;

        // External service: Clear Redis cache
        let _ = self.redis.delete_key(&format!("profile:user_id:{}", id).to_string().into()).await;

        // TODO: External service - Kafka event publishing
        // self.kafka_producer.send(...)

        Ok(true)
    }

    async fn list_users(
        &self,
        conn: &DatabaseTransaction,
        page: u64,
        page_size: u64,
    ) -> AppResult<Vec<UserSerializer>> {
        // Database: Fetch paginated users
        let users = user::user::Entity::list_users(conn, page, page_size).await?;
        let mut user_serializers = Vec::new();

        // Database: Load relationships for each user
        for user in users {
            if let Ok(Some(user_with_address)) = user::user::Entity::find_user_by_id(conn, user.id).await {
                user_serializers.push(UserSerializer::from(user_with_address));
            }
        }

        Ok(user_serializers)
    }

    async fn logout(&self, _conn: &DatabaseTransaction, user_id: i64) -> AppResult<bool> {
        // External service: Clear Redis cache (session invalidation)
        match self.redis.delete_key(&format!("profile:user_id:{user_id}").to_string().into()).await
        {
            Ok(_) => Ok(true),
            Err(err) => Err(AppError::BadRequestError(err.to_string())),
        }
    }
}
