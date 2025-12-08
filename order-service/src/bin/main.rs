use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use log::{error, info, LevelFilter};
use rand::rngs::OsRng;
use order_service::core::error::AppResult;
use order_service::core::http::server::AppServer;

#[tokio::main]
async fn main() -> AppResult<()> {
    env_logger::builder().filter_level(LevelFilter::Debug).format_target(true).init();

    info!("The initialization of Tracing was successful!");
    let config = CONFIG.clone();
    let server = AppServer::new(config).await?;
    let db = server.state.db.clone();
    let redis = server.state.redis.clone();
    info!("Starting server...");

    let server_task = tokio::spawn(async {
        if let Err(e) = server.run().await {
            error!("HTTP Server error: {:?}", e);
        }
    });

    let _server_result = tokio::join!(server_task);

    Ok(())
}
