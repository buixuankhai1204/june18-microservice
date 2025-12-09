use crate::presentation::user::user::{UserSerializer, CreateUserRequest, UpdateUserRequest, UserCreatedSerializer};
use crate::application::user::user_command::{RegisterUserCommand, VerifyEmailCommand, ResendVerificationEmailCommand};
use sea_orm::DatabaseTransaction;
use crate::infrastructure::error::AppResult;

pub trait UserServiceInterface: Send + Sync + 'static {
    async fn register_user(
        &self,
        conn: &DatabaseTransaction,
        command: RegisterUserCommand,
    ) -> AppResult<UserCreatedSerializer>;

    async fn verify_email(
        &self,
        conn: &DatabaseTransaction,
        command: VerifyEmailCommand,
    ) -> AppResult<bool>;

    async fn resend_verification_email(
        &self,
        conn: &DatabaseTransaction,
        command: ResendVerificationEmailCommand,
    ) -> AppResult<bool>;

    async fn create_user(
        &self,
        conn: &DatabaseTransaction,
        request: CreateUserRequest,
    ) -> AppResult<bool>;

    async fn update_user(
        &self,
        conn: &DatabaseTransaction,
        id: i64,
        request: UpdateUserRequest,
    ) -> AppResult<bool>;

    async fn get_profile(
        &self,
        conn: &DatabaseTransaction,
        user_id: i64,
    ) -> AppResult<UserSerializer>;

    async fn delete_user(
        &self,
        conn: &DatabaseTransaction,
        id: i64,
    ) -> AppResult<bool>;

    async fn list_users(
        &self,
        conn: &DatabaseTransaction,
        page: u64,
        page_size: u64,
    ) -> AppResult<Vec<UserSerializer>>;

    async fn logout(&self, conn: &DatabaseTransaction, id: i64) -> AppResult<bool>;
}
