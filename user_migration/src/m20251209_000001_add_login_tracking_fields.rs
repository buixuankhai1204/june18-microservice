use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add failed_login_attempts field
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::FailedLoginAttempts)
                            .integer()
                            .not_null()
                            .default(0)
                    )
                    .to_owned(),
            )
            .await?;

        // Add last_failed_login_at field
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::LastFailedLoginAt)
                            .timestamp()
                            .null()
                    )
                    .to_owned(),
            )
            .await?;

        // Add account_locked_until field
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::AccountLockedUntil)
                            .timestamp()
                            .null()
                    )
                    .to_owned(),
            )
            .await?;

        // Add last_login_at field
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::LastLoginAt)
                            .timestamp()
                            .null()
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop last_login_at field
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::LastLoginAt)
                    .to_owned(),
            )
            .await?;

        // Drop account_locked_until field
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::AccountLockedUntil)
                    .to_owned(),
            )
            .await?;

        // Drop last_failed_login_at field
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::LastFailedLoginAt)
                    .to_owned(),
            )
            .await?;

        // Drop failed_login_attempts field
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::FailedLoginAttempts)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    FailedLoginAttempts,
    LastFailedLoginAt,
    AccountLockedUntil,
    LastLoginAt,
}
