use crate::application::authen::authen_service_interface::AuthenServiceInterface;
use crate::infrastructure::persistence::redis_client::RedisConnectionPool;
use crate::infrastructure::third_party::token;
use crate::presentation::authen::authen::TokenResponse;
use rdkafka::producer::FutureProducer;
use sea_orm::{DatabaseTransaction, IntoActiveModel};
use std::sync::Arc;
use uuid::Uuid;
use crate::application::authen::authen_command::LoginByEmailCommand;
use crate::application::authen::claim::verify;
use crate::domain::user::user;
use crate::domain::user::user_repository_interface::UserRepositoryInterface;
use crate::infrastructure::error::{AppError, AppResult};

pub struct AuthenService {
    pub redis: Arc<RedisConnectionPool>,
    pub kafka_producer: Arc<FutureProducer>,
}

impl AuthenService {
    pub fn new(redis: Arc<RedisConnectionPool>, kafka_producer: Arc<FutureProducer>) -> Self {
        Self { redis, kafka_producer }
    }


}

impl AuthenServiceInterface for AuthenService {
    async fn login_by_email(
        &self,
        conn: &DatabaseTransaction,
        req: &LoginByEmailCommand
    ) -> AppResult<TokenResponse> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration as StdDuration;
        use crate::domain::user::events::user_logged_in::{UserLoggedInEvent, DeviceInfoEvent};
        use crate::presentation::authen::authen::UserInfo;

        // Find user by email
        let user_opt = user::Entity::find_user_by_email(conn, req.get_email()).await?;

        let mut user = user_opt.ok_or_else(||
            AppError::UnauthorizedError("Invalid email or password".to_string())
        )?;

        // Validate login attempt (check account status, lock status, failed login limit)
        if let Err(err) = user.validate_login_attempt() {
            return Err(err);
        }

        // Verify password
        let password_valid = match verify(
            req.get_password().to_string(),
            user.password.clone().unwrap_or_default()
        ).await {
            Ok(_) => true,
            Err(_) => false,
        };

        if !password_valid {
            // Handle failed login: increment counter and potentially lock account
            let updated_user = user.handle_failed_login();
            user::Entity::update_user(conn, updated_user.into_active_model()).await?;

            return Err(AppError::UnauthorizedError("Invalid email or password".to_string()));
        }

        // Handle successful login: reset failed attempts and update last_login_at
        user = user.handle_successful_login();
        user::Entity::update_user(conn, user.clone().into_active_model()).await?;

        // Generate session ID
        let session_id = Uuid::new_v4();

        // Store refresh token in Redis (7 days expiry)
        self.redis
            .set_key_with_expiry::<String>(
                &format!("refresh_token:session:{}", session_id),
                &session_id.to_string(),
                7 * 24 * 3600, // 7 days in seconds
            )
            .await
            .map_err(|err| AppError::BadRequestError(err.to_string()))?;

        // Create UserInfo for response
        let user_info = UserInfo {
            id: user.id.to_string(),
            email: user.email.clone(),
            full_name: format!("{} {}", user.first_name, user.last_name),
            role: match user.role {
                user::Role::CUSTOMER => "customer".to_string(),
                user::Role::ADMIN => "admin".to_string(),
            },
        };

        // Generate JWT tokens
        let token_response = token::service_generate_tokens(&user.id, &session_id, &user_info)?;

        // Publish UserLoggedIn event to Kafka
        let device_info_event = req.device_info.as_ref().map(|di| DeviceInfoEvent {
            user_agent: di.user_agent.clone(),
            ip_address: di.ip_address.clone(),
        });

        let event = UserLoggedInEvent::new(
            user.id,
            user.email.clone(),
            session_id.to_string(),
            device_info_event,
            chrono::Utc::now().naive_utc(),
        );

        let event_json = serde_json::to_string(&event)
            .map_err(|e| AppError::BadRequestError(format!("Failed to serialize event: {}", e)))?;

        let user_id_key = user.id.to_string();
        let kafka_record = FutureRecord::to(UserLoggedInEvent::topic_name())
            .payload(&event_json)
            .key(&user_id_key);

        match self.kafka_producer.send(kafka_record, StdDuration::from_secs(5)).await {
            Ok(_) => log::info!("UserLoggedIn event published for user_id: {}", user.id),
            Err(e) => log::error!("Failed to publish UserLoggedIn event: {:?}", e),
        }

        Ok(token_response)
    }

    async fn refresh_token(
        &self,
        _conn: &DatabaseTransaction,
        _refresh_token: &str,
    ) -> AppResult<TokenResponse> {
        // TODO: Implement refresh token logic
        Err(AppError::BadRequestError("Refresh token not implemented yet".to_string()))
    }

    async fn logout(&self, user_id: i64, user_uuid: &Uuid) -> AppResult<()> {
        // Delete refresh token from Redis
        self.redis
            .delete_key(&format!("refresh_token:session:{}", user_uuid))
            .await
            .map_err(|err| AppError::BadRequestError(err.to_string()))?;

        Ok(())
    }
}

