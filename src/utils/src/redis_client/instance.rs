use super::error::{RedisError, RedisResult};
use redis::{aio::ConnectionManager, Client};
use std::time::Duration;

/// Redis connection pool wrapper using redis crate
/// Uses ConnectionManager which automatically handles reconnection and is cheap to clone
#[derive(Clone)]
pub struct RedisConnectionPool {
    connection: ConnectionManager,
    pub key_prefix: String,
}

impl RedisConnectionPool {
    /// Create a new Redis connection pool from a Redis URL
    /// ConnectionManager automatically handles reconnection
    pub async fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let connection = ConnectionManager::new(client).await?;

        Ok(Self {
            connection,
            key_prefix: String::new(),
        })
    }

    /// Clone with a new key prefix
    /// ConnectionManager is cheap to clone and can be used concurrently
    pub fn with_prefix(&self, key_prefix: &str) -> Self {
        Self {
            connection: self.connection.clone(),
            key_prefix: key_prefix.to_string(),
        }
    }

    fn prefixed_key(&self, key: &str) -> String {
        if self.key_prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}:{}", self.key_prefix, key)
        }
    }

    pub async fn ping(&self) -> RedisResult<String> {
        let mut conn = self.connection.clone();
        let result: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(result)
    }

    pub async fn set(&self, key: &str, value: &str, expire: Duration) -> RedisResult<()> {
        use redis::AsyncCommands;
        let mut conn = self.connection.clone();
        let prefixed_key = self.prefixed_key(key);
        conn.set_ex(&prefixed_key, value, expire.as_secs()).await?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> RedisResult<Option<String>> {
        use redis::AsyncCommands;
        let mut conn = self.connection.clone();
        let prefixed_key = self.prefixed_key(key);
        let value: Option<String> = conn.get(&prefixed_key).await?;
        Ok(value)
    }

    pub async fn get_key<T>(&self, key: &str) -> RedisResult<Option<T>>
    where
        T: redis::FromRedisValue,
    {
        use redis::AsyncCommands;
        let mut conn = self.connection.clone();
        let prefixed_key = self.prefixed_key(key);
        let value: Option<T> = conn.get(&prefixed_key).await?;
        Ok(value)
    }

    pub async fn delete_key(&self, key: &str) -> RedisResult<bool> {
        use redis::AsyncCommands;
        let mut conn = self.connection.clone();
        let prefixed_key = self.prefixed_key(key);
        let deleted: bool = conn.del(&prefixed_key).await?;
        Ok(deleted)
    }

    pub async fn get_and_deserialize_key<T>(&self, key: &str, _type_name: &str) -> RedisResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let value: Option<String> = self.get(key).await?;
        let value_str = value.ok_or_else(|| {
            RedisError::NotFound(format!("Key not found: {}", key))
        })?;
        let deserialized: T = serde_json::from_str(&value_str)?;
        Ok(deserialized)
    }

    pub async fn serialize_and_set_key_with_expiry(
        &self,
        key: &str,
        value: &serde_json::Value,
        ttl_seconds: i64,
    ) -> RedisResult<()> {
        let value_str = serde_json::to_string(value)?;
        self.set(key, &value_str, Duration::from_secs(ttl_seconds as u64)).await
    }

    pub async fn scan(
        &self,
        pattern: &str,
        count: Option<usize>,
        _cursor: Option<u64>,
    ) -> RedisResult<Vec<String>> {
        let mut conn = self.connection.clone();
        let prefixed_pattern = self.prefixed_key(pattern);
        let mut all_keys = Vec::new();
        let mut cursor: u64 = 0;
        let scan_count = count.unwrap_or(100);

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&prefixed_pattern)
                .arg("COUNT")
                .arg(scan_count)
                .query_async(&mut conn)
                .await?;

            all_keys.extend(keys);
            cursor = new_cursor;

            if cursor == 0 {
                break;
            }

            if let Some(max_count) = count {
                if all_keys.len() >= max_count {
                    all_keys.truncate(max_count);
                    break;
                }
            }
        }

        Ok(all_keys)
    }

    pub async fn set_key_with_expiry<T>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: i64,
    ) -> RedisResult<()>
    where
        T: serde::Serialize,
    {
        let value_json = serde_json::to_value(value)?;
        self.serialize_and_set_key_with_expiry(key, &value_json, ttl_seconds).await
    }
}
