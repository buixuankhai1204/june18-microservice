pub use sea_orm_migration::prelude::*;

pub mod enum_type;

pub mod m20251126_142840_create_user_table;
pub mod m20251126_142841_create_address_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251126_142840_create_user_table::Migration),
            Box::new(m20251126_142841_create_address_table::Migration),
        ]
    }
}
