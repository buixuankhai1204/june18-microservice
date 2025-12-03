use crate::core::error::AppResult;
use crate::presentation::address::address::{AddressSerializer, CreateAddressRequest, UpdateAddressRequest};
use sea_orm::DatabaseTransaction;

pub trait AddressServiceInterface: Send + Sync + 'static {
    async fn create_address(
        &self,
        conn: &DatabaseTransaction,
        request: CreateAddressRequest,
    ) -> AppResult<bool>;

    async fn update_address(
        &self,
        conn: &DatabaseTransaction,
        id: i64,
        request: UpdateAddressRequest,
    ) -> AppResult<bool>;

    async fn get_address_by_id(
        &self,
        conn: &DatabaseTransaction,
        id: i64,
    ) -> AppResult<AddressSerializer>;

    async fn delete_address(
        &self,
        conn: &DatabaseTransaction,
        id: i64,
    ) -> AppResult<bool>;

    async fn get_addresses_by_user_id(
        &self,
        conn: &DatabaseTransaction,
        user_id: i64,
    ) -> AppResult<Vec<AddressSerializer>>;
}
