use crate::core::configure::db::DatabaseConfig;
use crate::core::configure::env::get_env_source;
use crate::core::configure::http::HttpClientConfig;
use crate::core::configure::kafka::KafkaConfig;
use crate::core::configure::redis::RedisConfig;
use crate::core::configure::secret::SecretConfig;
use crate::core::configure::server::ServerConfig;
use crate::util::dir::get_project_root;
use config::{ConfigError, Environment};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub profile: Profile,
    pub server: ServerConfig,
    pub db: DatabaseConfig,
    pub sentry: Sentry,
    pub redis: RedisConfig,
    pub secret: SecretConfig,
    pub http: HttpClientConfig,
    pub kafka: KafkaConfig,
}

impl AppConfig {
    pub fn read(profile: Profile) -> Result<Self, config::ConfigError> {
        let config_dir = get_settings_dir()?;
        let config = config::Config::builder()
            .add_source(config::File::from(config_dir.join(profile.filename())))
            .add_source(profile.env_source())
            .build()?;
        log::info!("Successfully read config profile: {profile}.");
        config.try_deserialize()
    }

    pub fn get_sentry_dsn(&self) -> &str {
        &self.sentry.dsn
    }
}

pub fn get_settings_dir() -> Result<std::path::PathBuf, ConfigError> {
    Ok(get_project_root().map_err(|e| ConfigError::Message(e.to_string()))?.join("settings"))
}

pub fn get_static_dir() -> Result<std::path::PathBuf, ConfigError> {
    Ok(get_project_root().map_err(|e| ConfigError::Message(e.to_string()))?.join("static"))
}

#[derive(
    Debug,
    strum::Display,
    strum::EnumString,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
)]
pub enum Profile {
    #[serde(rename = "dev")]
    #[strum(serialize = "dev")]
    Dev,
    #[serde(rename = "stag")]
    #[strum(serialize = "stag")]
    Stag,
    #[serde(rename = "prod")]
    #[strum(serialize = "prod")]
    Prod,
    #[serde(rename = "test")]
    #[strum(serialize = "test")]
    Test,
    #[serde(rename = "local")]
    #[strum(serialize = "local")]
    Local,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Sentry {
    pub dsn: String,
}

impl Profile {
    fn filename(&self) -> String {
        format!("{self}.toml")
    }

    fn env_source(&self) -> Environment {
        get_env_source(&format!("{}_APP", self.to_string().to_uppercase()))
    }
}
