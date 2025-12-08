//! Redis cache helper utilities
//!
//! This module provides utility functions and patterns for consistent Redis caching
//! across application services. It implements common patterns like read-through caching,
//! cache invalidation, and cache key generation.

use crate::core::error::{AppError, AppResult};
use crate::infrastructure::third_party::redis::lib::RedisConnectionPool;
use chrono::NaiveDateTime;
use log::{debug, warn};
use serde::{de::DeserializeOwned, Serialize};

/// Cache key builder for consistent key naming across the application
pub struct CacheKeyBuilder {
    parts: Vec<String>,
}

impl CacheKeyBuilder {
    /// Create a new cache key builder with entity type
    pub fn new(entity_type: &str) -> Self {
        Self { parts: vec![entity_type.to_string()] }
    }

    /// Add an identifier to the key (e.g., "id:123")
    pub fn with_id(mut self, id_type: &str, id_value: impl ToString) -> Self {
        self.parts.push(format!("{}:{}", id_type, id_value.to_string()));
        self
    }

    /// Add a filter parameter (e.g., "channel:5")
    pub fn with_filter(mut self, filter_name: &str, filter_value: impl ToString) -> Self {
        self.parts.push(format!("{}:{}", filter_name, filter_value.to_string()));
        self
    }

    /// Add a date parameter (e.g., "date:2025-01-01")
    pub fn with_date(mut self, date: NaiveDateTime) -> Self {
        self.parts.push(format!("date:{}", date.format("%Y-%m-%d")));
        self
    }

    /// Add pagination info (e.g., "page:0:size:10")
    pub fn with_pagination(mut self, page: i32, page_size: i32) -> Self {
        self.parts.push(format!("page:{}:size:{}", page, page_size));
        self
    }

    /// Add custom parameter
    pub fn with_param(mut self, key: &str, value: impl ToString) -> Self {
        self.parts.push(format!("{}:{}", key, value.to_string()));
        self
    }

    /// Build the final cache key
    pub fn build(self) -> String {
        self.parts.join(":")
    }
}

/// Read-through cache pattern: Try cache first, then fetch from source if miss
///
/// # Arguments
/// * `redis` - Redis connection pool
/// * `cache_key` - Cache key to use
/// * `ttl` - Time to live in seconds
/// * `fetch_fn` - Async function to fetch data on cache miss
///
/// # Example
/// ```rust
/// let user = read_through_cache(
///     &self.redis,
///     &CacheKeyBuilder::new("user").with_id("id", 123).build(),
///     3600,
///     || async { UserRepository::get_by_id(conn, 123).await }
/// ).await?;
/// ```
pub async fn read_through_cache<T, F, Fut>(
    redis: &RedisConnectionPool,
    cache_key: &str,
    ttl: i64,
    fetch_fn: F,
) -> AppResult<T>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = AppResult<T>>,
{
    // Try to get from cache
    match redis
        .get_and_deserialize_key::<T>(&cache_key.to_string().into(), std::any::type_name::<T>())
        .await
    {
        Ok(cached_value) => {
            debug!("Cache hit for key: {}", cache_key);
            Ok(cached_value)
        },
        Err(_) => {
            debug!("Cache miss for key: {}", cache_key);

            // Fetch from source
            let value = fetch_fn().await?;

            // Store in cache (fire and forget - don't fail if cache write fails)
            let cache_key_for_cache = cache_key.to_string();
            let value_clone = serde_json::to_value(&value).ok();

            if let Some(v) = value_clone {
                let redis_clone = redis.clone("");
                tokio::spawn(async move {
                    let cache_key_redis = cache_key_for_cache.clone();
                    if let Err(e) = redis_clone
                        .serialize_and_set_key_with_expiry(&cache_key_for_cache.into(), &v, ttl)
                        .await
                    {
                        warn!("Failed to cache data for key {}: {:?}", cache_key_redis, e);
                    }
                });
            }

            Ok(value)
        },
    }
}

/// Invalidate a single cache key
pub async fn invalidate_cache(redis: &RedisConnectionPool, cache_key: &str) -> AppResult<()> {
    match redis.delete_key(&cache_key.to_string().into()).await {
        Ok(_) => {
            debug!("Invalidated cache key: {}", cache_key);
            Ok(())
        },
        Err(e) => {
            warn!("Failed to invalidate cache key {}: {:?}", cache_key, e);
            // Don't fail the operation if cache invalidation fails
            Ok(())
        },
    }
}

/// Invalidate multiple cache keys by pattern
///
/// # Example
/// ```rust
/// // Invalidate all list caches for program_slots
/// invalidate_cache_pattern(&self.redis, "list:program_slots:*").await?;
/// ```
pub async fn invalidate_cache_pattern(redis: &RedisConnectionPool, pattern: &str) -> AppResult<()> {
    match redis.scan(&pattern.to_string().into(), Some(100), None).await {
        Ok(keys) => {
            if keys.is_empty() {
                debug!("No cache keys found for pattern: {}", pattern);
                return Ok(());
            }

            debug!("Found {} keys matching pattern: {}", keys.len(), pattern);

            for key in keys {
                let _ = redis.delete_key(&key.into()).await;
            }

            Ok(())
        },
        Err(e) => {
            warn!("Failed to scan cache pattern {}: {:?}", pattern, e);
            Ok(())
        },
    }
}

/// Invalidate multiple patterns at once
pub async fn invalidate_multiple_patterns(
    redis: &RedisConnectionPool,
    patterns: Vec<String>,
) -> AppResult<()> {
    for pattern in patterns {
        invalidate_cache_pattern(redis, &pattern).await?;
    }
    Ok(())
}

/// Cache list results with automatic key generation from query parameters
///
/// # Example
/// ```rust
/// let programs = cache_list_query(
///     &self.redis,
///     "program",
///     &query_params,
///     REDIS_TTL_LIST_SHORT,
///     || async { ProgramRepository::list(conn, query_params).await }
/// ).await?;
/// ```
pub async fn cache_list_query<T, F, Fut>(
    redis: &RedisConnectionPool,
    entity_type: &str,
    query_params: &crate::util::filter_and_pagination::PageQueryParam,
    ttl: i64,
    fetch_fn: F,
) -> AppResult<Vec<T>>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = AppResult<Vec<T>>>,
{
    let mut key_builder = CacheKeyBuilder::new(&format!("list:{}", entity_type))
        .with_pagination(
            query_params.page_num.unwrap_or(0) as i32,
            query_params.page_size.unwrap_or(10) as i32,
        );

    if let Some(start_date) = query_params.start_date {
        key_builder = key_builder.with_date(start_date);
    }

    if let Some(end_date) = query_params.end_date {
        key_builder = key_builder.with_param("end", end_date.format("%Y-%m-%d").to_string());
    }

    if let Some(q) = &query_params.q {
        key_builder = key_builder.with_param("q", q);
    }

    let cache_key = key_builder.build();

    read_through_cache(redis, &cache_key, ttl, fetch_fn).await
}

/// Get or compute a value with caching
///
/// This is useful for computed/aggregated data like statistics
pub async fn get_or_compute<T, F, Fut>(
    redis: &RedisConnectionPool,
    cache_key: &str,
    ttl: i64,
    compute_fn: F,
) -> AppResult<T>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = AppResult<T>>,
{
    read_through_cache(redis, cache_key, ttl, compute_fn).await
}

/// Batch fetch with caching using hash structure
///
/// Efficiently fetches multiple entities by ID, using cache where available
/// and fetching missing items from database
///
/// # Example
/// ```rust
/// let programs = batch_fetch_with_cache(
///     &self.redis,
///     "program:by_id",
///     &[1, 2, 3, 4, 5],
///     900,
///     |missing_ids| async move {
///         ProgramRepository::get_by_ids(conn, missing_ids).await
///     }
/// ).await?;
/// ```
pub async fn batch_fetch_with_cache<T, F, Fut>(
    redis: &RedisConnectionPool,
    hash_key: &str,
    ids: &[i64],
    ttl: u32,
    fetch_fn: F,
) -> AppResult<Vec<T>>
where
    T: Serialize + DeserializeOwned + Clone + std::fmt::Debug + Send + Sync + 'static,
    F: FnOnce(Vec<i64>) -> Fut,
    Fut: std::future::Future<Output = AppResult<Vec<(i64, T)>>>,
{
    let mut results: Vec<(i64, T)> = Vec::new();
    let mut missing_ids = Vec::new();

    // Try to get from cache
    for &id in ids {
        let field = id.to_string();
        match redis
            .get_hash_field_and_deserialize::<T>(
                &hash_key.to_string().into(),
                &field,
                std::any::type_name::<T>(),
            )
            .await
        {
            Ok(entity) => {
                debug!("Cache hit for {}:{}", hash_key, id);
                results.push((id, entity));
            },
            Err(_) => {
                debug!("Cache miss for {}:{}", hash_key, id);
                missing_ids.push(id);
            },
        }
    }

    // Fetch missing from source
    if !missing_ids.is_empty() {
        let fetched = fetch_fn(missing_ids).await?;

        // Cache the fetched entities (spawn to not block)
        let redis_clone = redis.clone("");
        let hash_key_owned = hash_key.to_string();
        let fetched_clone = fetched.clone();

        tokio::spawn(async move {
            for (id, entity) in fetched_clone {
                let field = id.to_string();
                if let Err(e) = redis_clone
                    .serialize_and_set_hash_field_if_not_exist(
                        &hash_key_owned.clone().into(),
                        &field,
                        &entity,
                        Some(ttl),
                    )
                    .await
                {
                    warn!("Failed to cache entity {}:{}: {:?}", hash_key_owned, id, e);
                }
            }
        });

        results.extend(fetched);
    }

    // Sort by original ID order and extract values
    let id_order: std::collections::HashMap<i64, usize> =
        ids.iter().enumerate().map(|(idx, &id)| (id, idx)).collect();

    results.sort_by_key(|(id, _)| id_order.get(id).copied().unwrap_or(usize::MAX));

    Ok(results.into_iter().map(|(_, entity)| entity).collect())
}

/// Update cache after entity modification
///
/// Use this pattern when you want to update cache instead of invalidating
pub async fn update_cache<T>(
    redis: &RedisConnectionPool,
    cache_key: &str,
    entity: &T,
    ttl: i64,
) -> AppResult<()>
where
    T: Serialize + std::fmt::Debug,
{
    match redis.serialize_and_set_key_with_expiry(&cache_key.to_string().into(), entity, ttl).await
    {
        Ok(_) => {
            debug!("Updated cache for key: {}", cache_key);
            Ok(())
        },
        Err(e) => {
            warn!("Failed to update cache for key {}: {:?}", cache_key, e);
            Ok(())
        },
    }
}

/// Safely wrap a cache operation to not fail the main operation
///
/// Use this for cache operations that should never cause the main operation to fail
pub async fn safe_cache_op<F, Fut>(op: F)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = AppResult<()>>,
{
    if let Err(e) = op().await {
        warn!("Cache operation failed (non-critical): {:?}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_builder() {
        let key = CacheKeyBuilder::new("user")
            .with_id("id", 123)
            .with_filter("status", "active")
            .build();

        assert_eq!(key, "user:id:123:status:active");
    }

    #[test]
    fn test_cache_key_builder_with_date() {
        let date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let key = CacheKeyBuilder::new("program_slot").with_id("channel", 5).with_date(date).build();

        assert_eq!(key, "program_slot:channel:5:date:2025-01-15");
    }

    #[test]
    fn test_cache_key_builder_pagination() {
        let key = CacheKeyBuilder::new("list:programs").with_pagination(2, 20).build();

        assert_eq!(key, "list:programs:page:2:size:20");
    }
}
