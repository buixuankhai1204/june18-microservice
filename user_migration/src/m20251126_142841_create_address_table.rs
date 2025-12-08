use sea_orm_migration::{prelude::*, schema::*};
use super::m20251126_142840_create_user_table::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Addresses::Table)
                    .if_not_exists()
                    .col(pk_auto(Addresses::Id))
                    .col(integer(Addresses::UserId))
                    .col(string_null(Addresses::Title))
                    .col(string(Addresses::AddressLine1))
                    .col(string_null(Addresses::AddressLine2))
                    .col(string(Addresses::Country))
                    .col(string(Addresses::City))
                    .col(string_null(Addresses::PostalCode))
                    .col(string_null(Addresses::Landmark))
                    .col(string_null(Addresses::PhoneNumber))
                    .col(timestamp_null(Addresses::CreatedAt))
                    .col(timestamp_null(Addresses::DeletedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_addresses_user_id")
                            .from(Addresses::Table, Addresses::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on user_id for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_addresses_user_id")
                    .table(Addresses::Table)
                    .col(Addresses::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Addresses::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Addresses {
    Table,
    Id,
    UserId,
    Title,
    AddressLine1,
    AddressLine2,
    Country,
    City,
    PostalCode,
    Landmark,
    PhoneNumber,
    CreatedAt,
    DeletedAt,
}
