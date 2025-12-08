use crate::core::configure::app::AppConfig;
use crate::core::error::{AppError, AppResult};
use crate::infrastructure::persistence::postgres::{DatabaseClient, DatabaseClientExt};
use crate::util;
use log::{error, info};
use order_migration::{Alias, Iden, IntoIden, SelectExpr, SelectStatement};
use sea_orm::SqlxError::Database;
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr, EntityTrait, QueryResult, QueryTrait, RuntimeErr, Statement};
use std::borrow::Cow;

async fn create_database(db: &DatabaseConnection, database_name: &str) -> AppResult {
    db.execute_unprepared(&format!("CREATE DATABASE {database_name}")).await?;
    log::info!("Create new database: {database_name}.");
    Ok(())
}

pub async fn setup_new_database(config: &mut AppConfig) -> AppResult<DatabaseClient> {
    info!("Setup new database for the test.");
    let db = DatabaseClient::build_from_config(config).await?;
    config.db.database_name =
        util::random::generate_random_string_with_prefix("uptop").to_lowercase();
    create_database(&db, &config.db.database_name).await?;
    Ok(db)
}

pub async fn drop_database(db: &DatabaseConnection, database_name: &str) -> AppResult {
    let drop_query = format!("DROP DATABASE {database_name};");
    let query = format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}';",
        database_name
    );
    db.execute_unprepared(&query).await?;
    db.execute_unprepared(&drop_query).await?;
    info!("Drop database: {database_name}.");
    Ok(())
}

//
// pub fn err_handling(err: &DbErr) -> AppError {
//     match err {
//         DbErr::Conn(err) => {
//             error!("Unhandled connection error: {:?}", err);
//             AppError::DatabaseErrorMessage { detail: "Unhandled connection error".to_string() }
//         },
//         DbErr::Query(error) => match error {
//             RuntimeErr::SqlxError(e) => match e {
//                 Database(e) => {
//                     info!("Database connection error: {:?}", e);
//                     match e.code() {
//                         Some(code) => match code {
//                             Cow::Borrowed("23505") => {
//                                 if e.is_unique_violation() {
//                                     let column_name: Vec<&str> =
//                                         e.constraint().unwrap().split("_").collect();
//                                     let message = format!(
//                                         "trường {} đã tồn tại, vui lòng nhập một teen khác!",
//                                         column_name[1].to_owned()
//                                     );
//                                     return AppError::DatabaseErrorMessage {
//                                         detail: message.to_string(),
//                                     };
//                                 } else {
//                                     error!("Unhandled unique violation error: {}", e.message());
//                                     return AppError::DatabaseErrorMessage {
//                                         detail: "Unhandled unique violation error".to_string(),
//                                     };
//                                 }
//                             },
//                             Cow::Borrowed("42P01") => {
//                                 error!("Table does not exist, check your schema");
//                                 return AppError::DatabaseErrorMessage {
//                                     detail: "Table does not exist".to_string(),
//                                 };
//                             },
//                             _ => {
//                                 error!("Unhandled database error code: {}", code);
//                                 return AppError::DatabaseErrorMessage {
//                                     detail: e.message().to_string(),
//                                 };
//                             },
//                         },
//                         _ => AppError::DatabaseErrorMessage { detail: e.message().to_string() },
//                     }
//                 },
//                 _ => {
//                     error!("Unhandled database connection error: {}", e);
//                     AppError::DatabaseErrorMessage {
//                         detail: "Unhandled database connection error".to_string(),
//                     }
//                 },
//             },
//             _ => {
//                 error!("Unhandled connection error: {:?}", err);
//                 AppError::DatabaseErrorMessage { detail: "Unhandled connection error".to_string() }
//             },
//         },
//         _ => {
//             error!("Unhandled database error: {:?}", err);
//             AppError::DatabaseErrorMessage { detail: "Unhandled database error".to_string() }
//         },
//     }
// }
