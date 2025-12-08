use super::user;
use crate::infrastructure::error::AppResult;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

#[async_trait]
pub trait UserRepositoryInterface: Send + Sync {
    async fn create_user(conn: &DatabaseTransaction, model: user::ActiveModelEx) -> AppResult<bool>;
    async fn update_user(conn: &DatabaseTransaction, model: user::ActiveModelEx) -> AppResult<bool>;
    async fn find_user_by_id(conn: &DatabaseTransaction, id: i64) -> AppResult<Option<user::ModelEx>>;
    async fn find_user_by_username(conn: &DatabaseTransaction, username: &str) -> AppResult<Option<user::ModelEx>>;
    async fn find_user_by_email(conn: &DatabaseTransaction, email: &str) -> AppResult<Option<user::ModelEx>>;
    async fn delete_user(conn: &DatabaseTransaction, id: i64) -> AppResult<()>;
    async fn username_exists(conn: &DatabaseTransaction, username: &str) -> AppResult<bool>;
    async fn email_exists(conn: &DatabaseTransaction, email: &str) -> AppResult<bool>;
    async fn phone_exists(conn: &DatabaseTransaction, phone: &str) -> AppResult<bool>;
    async fn list_users(conn: &DatabaseTransaction, page: u64, page_size: u64) -> AppResult<Vec<user::Model>>;
}
