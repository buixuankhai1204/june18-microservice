use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use log::{error, info, LevelFilter};
use product_service::core::error::AppResult;
use product_service::core::http::server::AppServer;
use product_service::util::constant::CONFIG;
use rand::rngs::OsRng;

fn generate_admin_password() -> String {
    let password = "admin123";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2.hash_password(password.as_bytes(), &salt).unwrap().to_string()
}

#[tokio::main]
async fn main() -> AppResult<()> {
    env_logger::builder().filter_level(LevelFilter::Debug).format_target(true).init();

    info!("The initialization of Tracing was successful!");
    let config = CONFIG.clone();
    let server = AppServer::new(config).await?;
    let db = server.state.db.clone();
    let redis = server.state.redis.clone();
    info!("Starting server...");

    println!("Admin password hash: {}", generate_admin_password());

    let server_task = tokio::spawn(async {
        if let Err(e) = server.run().await {
            error!("HTTP Server error: {:?}", e);
        }
    });

    let _server_result = tokio::join!(server_task);

    Ok(())
}
