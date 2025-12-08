use super::address;
use crate::infrastructure::error::AppResult;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;
use crate::domain::address::address::ActiveModelEx;

#[async_trait]
pub trait AddressRepositoryInterface: Send + Sync {
    async fn create_address(conn: &DatabaseTransaction, model: ActiveModelEx) -> AppResult<bool>;
    async fn update_address(conn: &DatabaseTransaction, model: ActiveModelEx) -> AppResult<bool>;
    async fn find_address_by_id(conn: &DatabaseTransaction, id: i64) -> AppResult<Option<address::ModelEx>>;
    async fn delete_address(conn: &DatabaseTransaction, id: i64) -> AppResult<()>;
    async fn find_addresses_by_user_id(conn: &DatabaseTransaction, user_id: i64) -> AppResult<Vec<address::ModelEx>>;
}
