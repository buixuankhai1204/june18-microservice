use crate::core::client::http::{ClientBuilder, HttpClient};
use crate::core::configure;
use crate::core::configure::app::Profile;
use jsonwebtoken::{DecodingKey, EncodingKey};
use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;

pub const MAX_RETRY: u32 = 10;
pub const ENV_PREFIX: &str = "APP";
pub const CODE_LEN: usize = 5;
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(120);
pub const EXPIRE_SESSION_CODE_SECS: Duration = Duration::from_secs(36000);
pub const EXPIRE_INVITATION_CODE_SECS: Duration = Duration::from_secs(86000);
pub const EXPIRE_BLOCKED_EMAIL_SECS: Duration = Duration::from_secs(300);
pub const EXPIRE_FORGET_PASS_CODE_SECS: Duration = Duration::from_secs(300);
pub const EXPIRE_BEARER_TOKEN_SECS: Duration = Duration::from_secs(36000);
pub const EXPIRE_REFRESH_TOKEN_SECS: Duration = Duration::from_secs(86400);
pub const QUEUE_EMPTY_DELAY_SECS: Duration = Duration::from_secs(60);
pub const COMPLETE_TASK_DELAY_SECS: Duration = Duration::from_secs(10);
pub const CHECK_EMAIL_MESSAGE: &str = "Please check you email.";
pub const AUTHORIZATION: &str = "Authorization";
pub const BEARER: &str = "Bearer";
pub const APP_DOMAIN: &str = "";
pub const APP_EMAIL_ADDR: &str = "";
pub const MINIMUM_DELAY_TIME: Duration = Duration::from_millis(120);

// Redis TTL Constants (in seconds)
pub const REDIS_TTL_USER_PROFILE: i64 = 86400; // 24 hours
pub const REDIS_TTL_EMPLOYEE: i64 = 86400; // 2 hours
pub const REDIS_TTL_DEPARTMENT: i64 = 86400; // 1 hour
pub const REDIS_TTL_CHANNEL: i64 = 86400; // 2 hours
pub const REDIS_TTL_CATEGORY: i64 = 86400; // 1 hour
pub const REDIS_TTL_POSITION: i64 = 86400; // 1 hour
pub const REDIS_TTL_PROGRAM: i64 = 86400; // 15 minutes
pub const REDIS_TTL_PROGRAM_SLOT: i64 = 3600; // 5 minutes
pub const REDIS_TTL_FRAME: i64 = 900; // 15 minutes
pub const REDIS_TTL_FRAME_CONFIG: i64 = 1800; // 30 minutes
pub const REDIS_TTL_PLAYLIST: i64 = 180; // 3 minutes
pub const REDIS_TTL_EDIT_REQUEST: i64 = 300; // 5 minutes
pub const REDIS_TTL_APPROVAL_FORM: i64 = 300; // 5 minutes
pub const REDIS_TTL_APPROVAL_FORM_CONFIG: i64 = 3600; // 1 hour
pub const REDIS_TTL_APPROVAL_UNIT: i64 = 1800; // 30 minutes
pub const REDIS_TTL_CUT_CONFIG: i64 = 1800; // 30 minutes
pub const REDIS_TTL_ADVERTISEMENT: i64 = 900; // 15 minutes
pub const REDIS_TTL_SOCIALIZE_FRAME: i64 = 900; // 15 minutes
pub const REDIS_TTL_REPEAT_FRAME_CONFIG: i64 = 1800; // 30 minutes
pub const REDIS_TTL_LIST_SHORT: i64 = 300; // 5 minutes - for frequently changing lists
pub const REDIS_TTL_LIST_MEDIUM: i64 = 600; // 10 minutes - for date-range queries
pub const REDIS_TTL_LIST_FILTERED: i64 = 180; // 3 minutes - for filtered queries
pub const REDIS_TTL_SEARCH_RESULT: i64 = 120; // 2 minutes - for search results
pub const REDIS_TTL_STATISTICS: i64 = 300; // 5 minutes - for computed stats
pub const REDIS_TTL_DEFAULT: i64 = 3600; // 1 hour - default TTL for other data

// pub static IMAGES_PATH: Lazy<PathBuf> = Lazy::new(|| get_static_dir().unwrap().join("images"));
// pub static APP_IMAGE: Lazy<PathBuf> = Lazy::new(|| get_static_dir().unwrap().join("images/logo.jpg"));

pub static CONFIG: LazyLock<configure::app::AppConfig> =
    LazyLock::new(|| configure::app::AppConfig::read(Profile::Local).unwrap());

pub static HTTP: LazyLock<Client> =
    LazyLock::new(|| HttpClient::build_from_config(&CONFIG).unwrap());
// pub static REDIS: Lazy<RedisClient> = Lazy::new(|| RedisClient::build_from_config(&CONFIG).unwrap());
// pub static EMAIL: Lazy<EmailClient> = Lazy::new(|| EmailClient::build_from_config(&CONFIG).unwrap());
pub static REFRESH_TOKEN_ENCODE_KEY: LazyLock<EncodingKey> = LazyLock::new(|| {
    let key = CONFIG.secret.read_private_refresh_key().unwrap();
    EncodingKey::from_rsa_pem(key.as_bytes()).unwrap()
});

pub static REFRESH_TOKEN_DECODE_KEY: LazyLock<DecodingKey> = LazyLock::new(|| {
    let key = CONFIG.secret.read_public_refresh_key().unwrap();
    DecodingKey::from_rsa_pem(key.as_bytes()).unwrap()
});
pub static ACCESS_TOKEN_ENCODE_KEY: LazyLock<EncodingKey> = LazyLock::new(|| {
    let key = CONFIG.secret.read_private_access_key().unwrap();
    EncodingKey::from_rsa_pem(key.as_bytes()).unwrap()
});

pub static ACCESS_TOKEN_DECODE_KEY: LazyLock<DecodingKey> = LazyLock::new(|| {
    let key = CONFIG.secret.read_public_access_key().unwrap();
    DecodingKey::from_rsa_pem(key.as_bytes()).unwrap()
});
// pub static API_DOC: LazyLock<utoipa::openapi::OpenApi> = LazyLock::new(ApiDoc::openapi);
// pub static TEMPLATE_ENGIN: Lazy<TemplateEngine> = Lazy::new(|| {
//     let path = get_static_dir().unwrap().join("template/**/*").into_os_string().into_string().unwrap();
//     TemplateEngine::new(&path).unwrap()
// });
