use crate::core::error::AppResult;
use crate::presentation::user::user::{UserSerializer, CreateUserRequest, UpdateUserRequest};
use sea_orm::DatabaseTransaction;

pub trait UserServiceInterface: Send + Sync + 'static {
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
