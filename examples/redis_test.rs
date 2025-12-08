/// Example test for Redis client
/// Run with: cargo run --example redis_test
///
/// Make sure Redis is running on localhost:6379

use api_gateway::core::configure::app::AppConfig;
use api_gateway::infrastructure::persistence::redis_client::RedisConnectionPool;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("Loading configuration...");
    let config = AppConfig::from_env()?;

    println!("Connecting to Redis at {}...", config.redis.get_url());
    let redis = RedisConnectionPool::new(&config).await?;

    // Test 1: Ping
    println!("\n=== Test 1: Ping ===");
    let pong = redis.ping().await?;
    println!("✓ Ping response: {}", pong);

    // Test 2: Set and Get
    println!("\n=== Test 2: Set and Get ===");
    redis.set("test:key", "test_value", Duration::from_secs(60)).await?;
    println!("✓ Set key 'test:key' = 'test_value'");

    let value = redis.get("test:key").await?;
    println!("✓ Get key 'test:key' = {:?}", value);
    assert_eq!(value, Some("test_value".to_string()));

    // Test 3: TTL
    println!("\n=== Test 3: TTL ===");
    let ttl = redis.ttl("test:key").await?;
    println!("✓ TTL for 'test:key' = {} seconds", ttl);
    assert!(ttl > 0 && ttl <= 60);

    // Test 4: Exists
    println!("\n=== Test 4: Exists ===");
    let exists = redis.exists("test:key").await?;
    println!("✓ Key 'test:key' exists: {}", exists);
    assert!(exists);

    // Test 5: Delete
    println!("\n=== Test 5: Delete ===");
    let deleted = redis.delete("test:key").await?;
    println!("✓ Deleted 'test:key': {}", deleted);
    assert!(deleted);

    let exists_after = redis.exists("test:key").await?;
    println!("✓ Key 'test:key' exists after delete: {}", exists_after);
    assert!(!exists_after);

    // Test 6: JSON serialization
    println!("\n=== Test 6: JSON Serialization ===");
    let json_value = serde_json::json!({
        "name": "John Doe",
        "age": 30,
        "active": true
    });
    redis
        .serialize_and_set_key_with_expiry("test:json", &json_value, 120)
        .await?;
    println!("✓ Set JSON data");

    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Person {
        name: String,
        age: u32,
        active: bool,
    }

    let person: Person = redis.get_and_deserialize_key("test:json", "Person").await?;
    println!("✓ Retrieved and deserialized: {:?}", person);
    assert_eq!(person.name, "John Doe");
    assert_eq!(person.age, 30);
    assert_eq!(person.active, true);

    // Test 7: Key prefix
    println!("\n=== Test 7: Key Prefix ===");
    let redis_prefixed = redis.clone_with_prefix("myapp");
    redis_prefixed.set("user:1", "Alice", Duration::from_secs(60)).await?;
    println!("✓ Set prefixed key 'myapp:user:1'");

    let value = redis_prefixed.get("user:1").await?;
    println!("✓ Get prefixed key = {:?}", value);
    assert_eq!(value, Some("Alice".to_string()));

    // Verify the actual key in Redis has the prefix
    let direct_value = redis.get("myapp:user:1").await?;
    println!("✓ Direct get 'myapp:user:1' = {:?}", direct_value);
    assert_eq!(direct_value, Some("Alice".to_string()));

    // Cleanup
    redis.delete("myapp:user:1").await?;
    redis.delete("test:json").await?;

    println!("\n✅ All tests passed!");

    Ok(())
}
