//! Interface for secret retrieval from the AWS Secret Manager service.;

use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_secretsmanager::{Client, Region};
use serde_json::{from_str, Map, Value};

/// Objects that manages the retrieval of secrets.
#[derive(Clone, Debug)]
pub struct Secrets {
    client: Client,
}

impl Secrets {
    /// Creates new instance of secrets manager.
    pub async fn new(region: String) -> Result<Self> {
        let region_provider = RegionProviderChain::first_try(Region::new(region));

        let shared_config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&shared_config);

        Ok(Self { client })
    }

    /// Fetches secret from the AWS Secret Manager.
    pub async fn get(&self, name: &str) -> Result<Value> {
        let resp = self
            .client
            .get_secret_value()
            .secret_id(name)
            .send()
            .await?;

        // Non-existing secrets are replaced with empty strings. This may be confusing.
        // Return error if this causes issues.
        let secret = resp.secret_string().unwrap_or("");
        Ok(Value::from(secret))
    }

    /// Fetches secret from the AWS Secret Manager returning as Value.
    pub async fn get_as_map(&self, name: &str) -> Result<Value> {
        let secret = self.get(name).await?;
        let value = from_str(secret.as_str().unwrap())?;
        Ok(value)
    }
}
