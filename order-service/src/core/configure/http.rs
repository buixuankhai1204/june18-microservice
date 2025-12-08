use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize};

pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Duration::from_secs(u64::deserialize(deserializer)?))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HttpClientConfig {
    #[serde(deserialize_with = "deserialize_duration")]
    pub timeout: Duration,
}
