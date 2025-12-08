use crate::application::authen::authen_service_interface::AuthenServiceInterface;
use crate::infrastructure::persistence::redis_client::RedisConnectionPool;
use crate::infrastructure::third_party::token;
use crate::presentation::authen::authen::TokenResponse;
use rdkafka::producer::FutureProducer;
use sea_orm::DatabaseTransaction;
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
        // Find user by username
        let user_res = match user::Entity::find_user_by_username(
            conn,
            req.get_username(),
        ).await
        {
            Ok(result) => result.ok_or(
                AppError::BadRequestError("User not found".to_string())
            )?,
            Err(err) => {
                return Err(err);
            },
        };

        match verify(req.get_password().to_string(), user_res.password.unwrap_or_default()).await {
            Ok(_) => (),
            Err(err) => return Err(err),
        }

        let user_uuid = Uuid::new_v4();
        self.redis
            .set_key_with_expiry::<String>(
                &format!("profile:user_id:{}", user_res.id),
                &user_uuid.to_string(),
                3600,
            )
            .await
            .map_err(|err| AppError::BadRequestError(err.to_string()))?;

        let res = match token::service_generate_tokens(&user_res.id, &user_uuid) {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        Ok(res)
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
        self.redis
            .delete_key(&format!("profile:user_id:{}", user_id))
            .await
            .map_err(|err| AppError::BadRequestError(err.to_string()))?;

        Ok(())
    }
}

