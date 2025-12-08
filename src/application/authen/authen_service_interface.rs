use crate::presentation::authen::authen::TokenResponse;
use sea_orm::DatabaseTransaction;
use uuid::Uuid;
use crate::application::authen::authen_command::LoginByEmailCommand;
use crate::infrastructure::error::AppResult;

pub trait AuthenServiceInterface: Send + Sync + 'static {
    async fn login_by_email(
        &self,
        conn: &DatabaseTransaction,
        login_by_email_command: &LoginByEmailCommand
    ) -> AppResult<TokenResponse>;

    async fn refresh_token(
        &self,
        conn: &DatabaseTransaction,
        refresh_token: &str,
    ) -> AppResult<TokenResponse>;

    async fn logout(
        &self,
        user_id: i64,
        user_uuid: &Uuid,
    ) -> AppResult<()>;
}

