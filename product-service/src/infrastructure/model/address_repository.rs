use crate::core::error::{AppError, AppResult};
use crate::domain::address::address::{ActiveModel, ActiveModelEx, Column, Entity, Model, ModelEx};
use crate::domain::address::address_repository_interface::AddressRepositoryInterface;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityLoaderTrait, EntityTrait, ExprTrait, QueryFilter, Set};

#[async_trait]
impl AddressRepositoryInterface for Entity {
    async fn create_address(conn: &DatabaseTransaction, model: ActiveModelEx) -> AppResult<bool> {
        let _address = model
            .insert(conn)
            .await
            .map_err(|e| AppError::DatabaseError(e));
        Ok(true)
    }

    async fn update_address(conn: &DatabaseTransaction, model: ActiveModelEx) -> AppResult<bool> {
        // Convert Model to ActiveModel in infrastructure layer
        let _address = model
            .update(conn)
            .await
            .map_err(|e| AppError::DatabaseError(e));
        Ok(true)
    }

    async fn find_address_by_id(conn: &DatabaseTransaction, id: i64) -> AppResult<Option<ModelEx>> {
        let address = Entity::load()
            .filter_by_id(id)
            .one(conn)
            .await?;
        Ok(address)
    }

    async fn delete_address(conn: &DatabaseTransaction, id: i64) -> AppResult<()> {
        let address = Entity::find_by_id(id).one(conn).await?.ok_or_else(|| {
            AppError::EntityNotFoundError {
                detail: format!("Address with id {} not found", id),
            }
        })?;

        let mut address: ActiveModel = address.into();
        address.is_deleted = Set(true);
        address.deleted_at = Set(Some(chrono::Utc::now().naive_utc()));
        address.update(conn).await?;
        Ok(())
    }

    async fn find_addresses_by_user_id(
        conn: &DatabaseTransaction,
        user_id: i64,
    ) -> AppResult<Vec<ModelEx>> {
        let addresses = Entity::load()
            .filter(
                Column::UserId
                    .eq(user_id)
                    .and(Column::IsDeleted.eq(false)),
            )
            .all(conn)
            .await
            .map_err(|e| AppError::DatabaseError(e));

        (addresses)
    }
}
