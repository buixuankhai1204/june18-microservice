use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_auto(Users::Id))
                    .col(string_null(Users::Avatar))
                    .col(string(Users::FirstName))
                    .col(string(Users::LastName))
                    .col(string_uniq(Users::Username))
                    .col(string_uniq(Users::Email))
                    .col(string_null(Users::Password))
                    .col(date_null(Users::BirthOfDate))
                    .col(string_null(Users::PhoneNumber))
                    .col(string(Users::Status).default("active".to_string()))
                    .col(boolean(Users::IsDeleted).default(false))
                    .col(timestamp_null(Users::CreatedAt))
                    .col(timestamp_null(Users::DeletedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Users {
    Table,
    Id,
    Avatar,
    FirstName,
    LastName,
    Username,
    Email,
    Password,
    BirthOfDate,
    PhoneNumber,
    Status,
    IsDeleted,
    CreatedAt,
    DeletedAt,
}
