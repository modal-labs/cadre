//! Interface for secret retrieval from the AWS Secret Manager service.;

use anyhow::{Context, Result};
use aws_sdk_secretsmanager::Client;
use aws_types::sdk_config::SdkConfig;
use serde_json::Value;

/// Objects that manages the retrieval of secrets.
#[derive(Clone, Debug)]
pub struct Secrets {
    client: Client,
}

impl Secrets {
    /// Creates new instance of secrets manager.
    pub async fn new(aws_config: &SdkConfig) -> Result<Self> {
        let client = Client::new(aws_config);
        Ok(Self { client })
    }

    /// Fetches a secret from the AWS Secret Manager.
    pub async fn get(&self, name: &str) -> Result<Value> {
        let resp = self
            .client
            .get_secret_value()
            .secret_id(name)
            .send()
            .await?;

        let secret = resp.secret_string().context("missing secret string")?;
        Ok(serde_json::from_str(secret)?)
    }
}
