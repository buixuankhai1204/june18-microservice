use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityLoaderTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use crate::infrastructure::error::AppResult;
use crate::domain::user::user::{ActiveModel, ActiveModelEx, Model, ModelEx};
use crate::domain::user::user_repository_interface::UserRepositoryInterface;
use crate::domain::{address, user};

#[async_trait]
impl UserRepositoryInterface for user::user::Entity {
    async fn create_user(conn: &DatabaseTransaction, model: ActiveModelEx) -> AppResult<bool> {
        // Convert Model to ActiveModel in infrastructure layer


        let user = model.insert(conn).await.map_err(
            |e| e,
        )?;

        Ok(true)
    }

    async fn update_user(conn: &DatabaseTransaction, model: ActiveModelEx) -> AppResult<bool> {
        let _user = model.update(conn).await?;
        Ok(true)
    }

    async fn find_user_by_id(conn: &DatabaseTransaction, id: i64) -> AppResult<Option<ModelEx>> {
        let user = user::user::Entity::load()
            .filter_by_id(id)
            .with(address::address::Entity)
            .one(conn)
            .await?;
        Ok(user)
    }

    async fn find_user_by_username(
        conn: &DatabaseTransaction,
        username: &str,
    ) -> AppResult<Option<ModelEx>> {
        let user = user::user::Entity::load()
            .filter(user::user::Column::Username.eq(username))
            .with(address::address::Entity)
            .one(conn)
            .await?;
        Ok(user)
    }

    async fn find_user_by_email(
        conn: &DatabaseTransaction,
        email: &str,
    ) -> AppResult<Option<ModelEx>> {
        let user = user::user::Entity::load()
            .filter(user::user::Column::Email.eq(email))
            .with(address::address::Entity)
            .one(conn)
            .await?;
        Ok(user)
    }

    async fn delete_user(conn: &DatabaseTransaction, id: i64) -> AppResult<()> {
        use sea_orm::Set;
        let user = user::user::Entity::find_by_id(id)
            .one(conn)
            .await?
            .ok_or_else(|| crate::infrastructure::error::AppError::EntityNotFoundError {
                detail: format!("User with id {} not found", id),
            })?;

        let mut user: ActiveModel = user.into();
        user.is_deleted = Set(true);
        user.deleted_at = Set(Some(chrono::Utc::now().naive_utc()));
        user.update(conn).await?;
        Ok(())
    }

    async fn username_exists(conn: &DatabaseTransaction, username: &str) -> AppResult<bool> {
        use sea_orm::EntityTrait;
        let count = user::user::Entity::find()
            .filter(user::user::Column::Username.eq(username))
            .filter(user::user::Column::IsDeleted.eq(false))
            .count(conn)
            .await?;
        Ok(count > 0)
    }

    async fn email_exists(conn: &DatabaseTransaction, email: &str) -> AppResult<bool> {
        use sea_orm::EntityTrait;
        let count = user::user::Entity::find()
            .filter(user::user::Column::Email.eq(email))
            .filter(user::user::Column::IsDeleted.eq(false))
            .count(conn)
            .await?;
        Ok(count > 0)
    }

    async fn phone_exists(conn: &DatabaseTransaction, phone: &str) -> AppResult<bool> {
        use sea_orm::EntityTrait;
        let count = user::user::Entity::find()
            .filter(user::user::Column::PhoneNumber.eq(phone))
            .filter(user::user::Column::IsDeleted.eq(false))
            .count(conn)
            .await?;
        Ok(count > 0)
    }

    async fn list_users(
        conn: &DatabaseTransaction,
        page: u64,
        page_size: u64,
    ) -> AppResult<Vec<Model>> {
        use sea_orm::{EntityTrait, PaginatorTrait};
        let users = user::user::Entity::find()
            .filter(user::user::Column::IsDeleted.eq(false))
            .paginate(conn, page_size)
            .fetch_page(page)
            .await?;
        Ok(users)
    }
}
