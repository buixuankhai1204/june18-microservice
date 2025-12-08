use crate::infrastructure::third_party::redis::errors;
use crate::infrastructure::third_party::redis::errors::CustomResult;
use crate::infrastructure::third_party::redis::types::RedisSettings;
use error_stack::ResultExt;
use fred::interfaces::ClientLike;
pub use fred::interfaces::PubsubInterface;
use std::sync::{atomic, Arc};

pub struct RedisConnectionPool {
    pub pool: Arc<fred::prelude::RedisPool>,
    pub key_prefix: String,
    pub config: Arc<RedisConfig>,
    pub is_redis_available: Arc<atomic::AtomicBool>,
}

pub struct RedisClient {
    inner: fred::prelude::RedisClient,
}

impl std::ops::Deref for RedisClient {
    type Target = fred::prelude::RedisClient;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl RedisClient {
    pub async fn new(
        config: fred::types::RedisConfig,
        reconnect_policy: fred::types::ReconnectPolicy,
        perf: fred::types::PerformanceConfig,
    ) -> CustomResult<Self, errors::RedisError> {
        let client =
            fred::prelude::RedisClient::new(config, Some(perf), None, Some(reconnect_policy));
        client.connect();
        client.wait_for_connect().await.change_context(errors::RedisError::RedisConnectionError)?;
        Ok(Self { inner: client })
    }
}

impl RedisConnectionPool {
    /// Create a new Redis connection
    pub async fn new(conf: &RedisSettings) -> CustomResult<Self, errors::RedisError> {
        let redis_connection_url = match conf.cluster_enabled {
            // Fred relies on this format for specifying cluster where the host port is ignored & only query parameters are used for node addresses
            // redis-cluster://username:password@host:port?node=bar.com:30002&node=baz.com:30003
            true => format!(
                "redis-cluster://{}:{}?{}",
                conf.host,
                conf.port,
                conf.cluster_urls.iter().flat_map(|url| vec!["&", url]).skip(1).collect::<String>()
            ),
            false => format!(
                "redis://{}:{}", //URI Schema
                conf.host, conf.port,
            ),
        };
        let mut config = fred::types::RedisConfig::from_url(&redis_connection_url)
            .change_context(errors::RedisError::RedisConnectionError)?;

        let perf = fred::types::PerformanceConfig {
            auto_pipeline: conf.auto_pipeline,
            default_command_timeout: std::time::Duration::from_secs(conf.default_command_timeout),
            max_feed_count: conf.max_feed_count,
            backpressure: fred::types::BackpressureConfig {
                disable_auto_backpressure: conf.disable_auto_backpressure,
                max_in_flight_commands: conf.max_in_flight_commands,
                policy: fred::types::BackpressurePolicy::Drain,
            },
        };

        let connection_config =
            fred::types::ConnectionConfig { ..fred::types::ConnectionConfig::default() };

        if !conf.use_legacy_version {
            config.version = fred::types::RespVersion::RESP3;
        }
        config.tracing = fred::types::TracingConfig::new(true);
        config.blocking = fred::types::Blocking::Error;
        let reconnect_policy = fred::types::ReconnectPolicy::new_constant(
            conf.reconnect_max_attempts,
            conf.reconnect_delay,
        );

        let pool = fred::prelude::RedisPool::new(
            config,
            Some(perf),
            Some(connection_config),
            Some(reconnect_policy),
            conf.pool_size,
        )
        .change_context(errors::RedisError::RedisConnectionError)?;

        pool.connect();
        pool.wait_for_connect().await.change_context(errors::RedisError::RedisConnectionError)?;

        let config = RedisConfig::from(conf);

        Ok(Self {
            pool: Arc::new(pool),
            config: Arc::new(config),
            is_redis_available: Arc::new(atomic::AtomicBool::new(true)),
            key_prefix: String::default(),
        })
    }
    pub fn clone(&self, key_prefix: &str) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
            key_prefix: key_prefix.to_string(),
            config: Arc::clone(&self.config),
            is_redis_available: Arc::clone(&self.is_redis_available),
        }
    }
}

pub struct RedisConfig {
    pub(crate) default_ttl: u32,
    pub(crate) default_stream_read_count: u64,
    pub(crate) default_hash_ttl: u32,
}

impl From<&RedisSettings> for RedisConfig {
    fn from(config: &RedisSettings) -> Self {
        Self {
            default_ttl: config.default_ttl,
            default_stream_read_count: config.stream_read_count,
            default_hash_ttl: config.default_hash_ttl,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_redis_error() {
        let x = errors::RedisError::ConsumerGroupClaimFailed.to_string();

        assert_eq!(x, "Failed to set Redis stream message owner".to_string())
    }
}
