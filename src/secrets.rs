//! Interface for secret retrieval from the AWS Secret Manager service.;

use std::sync::Arc;

use anyhow::{Context, Result};
use aws_sdk_secretsmanager::Client;
use aws_types::sdk_config::SdkConfig;
use cached::{Cached, TimedCache};
use parking_lot::Mutex;
use serde_json::Value;

/// Objects that manages the retrieval of secrets.
#[derive(Clone, Debug)]
pub struct Secrets {
    client: Option<Client>,
    cache: Arc<Mutex<TimedCache<String, Value>>>,
}

impl Secrets {
    /// Creates a new instance of secrets manager.
    pub fn new(aws_config: &SdkConfig) -> Self {
        let client = Client::new(aws_config);
        Self {
            client: Some(client),
            cache: Arc::new(Mutex::new(TimedCache::with_lifespan(60))),
        }
    }

    /// Creates a new instance with no backing secrets manager.
    pub fn new_test() -> Self {
        Self {
            client: None,
            cache: Arc::new(Mutex::new(TimedCache::with_lifespan(60))),
        }
    }

    /// Fetches a secret from the AWS Secret Manager.
    pub async fn get(&self, name: &str) -> Result<Value> {
        let name = name.to_string();
        if let Some(value) = self.cache.lock().cache_get(&name) {
            return Ok(value.clone());
        }

        let resp = self
            .client
            .as_ref()
            .context("AWS secrets client is missing")?
            .get_secret_value()
            .secret_id(&name)
            .send()
            .await?;

        let secret = resp.secret_string().context("missing secret string")?;
        let value: Value = serde_json::from_str(secret)?;
        self.cache.lock().cache_set(name, value.clone());
        Ok(value)
    }
}
