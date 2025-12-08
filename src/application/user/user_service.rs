use crate::infrastructure::persistence::redis_client::RedisConnectionPool;
use crate::application::user::user_service_interface::UserServiceInterface;
use crate::application::user::user_command::RegisterUserCommand;
use crate::domain::user::user_repository_interface::UserRepositoryInterface;
use crate::presentation::user::user::{UserSerializer, CreateUserRequest, UpdateUserRequest, UserCreatedSerializer};
use crate::api::domain::business_rule_interface::BusinessRuleInterface;
use crate::domain::user::rules::*;
use log::error;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use sea_orm::{DatabaseTransaction, IntoActiveModel, Set};
use std::sync::Arc;
use std::time::Duration;
use crate::application::authen::claim::hash;
use crate::domain::user;
use crate::domain::user::events::user_registered::UserRegisteredEvent;
use crate::domain::user::verification::generate_verification_token;
use crate::infrastructure::error::{AppError, AppResult};

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
    async fn register_user(
        &self,
        conn: &DatabaseTransaction,
        command: RegisterUserCommand,
    ) -> AppResult<UserCreatedSerializer> {
        // Business Rule: Email must be valid
        EmailMustBeValid { email: command.email.clone() }.check_broken()?;

        // Business Rule: Email must be unique
        let email_is_unique = !user::user::Entity::email_exists(conn, &command.email).await?;
        EmailMustBeUnique { is_unique: email_is_unique }.check_broken()?;

        // Business Rule: Password must meet requirements
        PasswordMustMeetRequirements { password: command.password.clone() }.check_broken()?;

        // Business Rule: Full name must be valid
        FullNameMustBeValid { full_name: command.full_name.clone() }.check_broken()?;

        // Business Rule: Phone must be valid and unique if provided
        if let Some(ref phone) = command.phone_number {
            PhoneMustBeValid { phone: phone.clone() }.check_broken()?;

            let phone_is_unique = !user::user::Entity::phone_exists(conn, phone).await?;
            PhoneMustBeUnique { is_unique: phone_is_unique }.check_broken()?;
        }

        // Business Rule: User must be at least 13 years old
        UserMustBeAtLeastAge {
            date_of_birth: command.date_of_birth,
            minimum_age: 13,
        }.check_broken()?;

        // Create user model
        let mut user = user::user::ModelEx::create_user_for_registration(
            command.email.clone(),
            command.password.clone(),
            command.full_name.clone(),
            command.phone_number.clone(),
            command.date_of_birth,
        )?;

        // Hash password using argon2 (salt rounds: 10 equivalent)
        let hashed_password = hash(command.password.clone()).await?;
        user.password = Some(hashed_password);

        // Generate verification token (expire: 24h)
        let (verification_token, token_expiry) = generate_verification_token();
        user.verification_token = Some(verification_token.clone());
        user.verification_token_expiry = Some(token_expiry);

        // Persist user to database
        let mut active_user = user.into_active_model();
        active_user.id = sea_orm::ActiveValue::NotSet;

        let created_user = active_user.insert(conn).await?;

        // Publish UserRegistered event to Kafka
        let event = UserRegisteredEvent::new(
            created_user.id,
            created_user.email.clone(),
            format!("{} {}", created_user.first_name, created_user.last_name),
            verification_token.clone(),
            created_user.created_at.unwrap_or_else(|| chrono::Utc::now().naive_utc()),
        );

        let event_json = serde_json::to_string(&event)
            .map_err(|e| AppError::BadRequestError(format!("Failed to serialize event: {}", e)))?;

        let kafka_record = FutureRecord::to(UserRegisteredEvent::topic_name())
            .payload(&event_json)
            .key(&created_user.id.to_string());

        // Send event asynchronously
        match self.kafka_producer.send(kafka_record, Duration::from_secs(5)).await {
            Ok(_) => log::info!("UserRegistered event published for user_id: {}", created_user.id),
            Err(e) => log::error!("Failed to publish UserRegistered event: {:?}", e),
        }

        // Return response
        Ok(UserCreatedSerializer {
            user_id: created_user.id.to_string(),
            email: created_user.email,
            message: "Please check your email to verify account".to_string(),
        })
    }

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
        let hashed_password = hash(request.password.clone()).await?;

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
        // let _ = self.redis..delete_key(&format!("profile:user_id:{}", id).to_string().into()).await;

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
                &format!("profile:user_id:{}", user_id),
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
                                &format!("profile:user_id:{}", user_id),
                                &serde_json::to_value(&profile).unwrap_or_default(),
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
        let _ = self.redis.delete_key(&format!("profile:user_id:{}", id)).await;

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
        self.redis
            .delete_key(&format!("profile:user_id:{user_id}"))
            .await
            .map_err(|err| AppError::BadRequestError(err.to_string()))?;
        Ok(true)
    }
}
