//! Interface for secret retrieval from the AWS Secret Manager service.;

use anyhow::{Context, Result};
use aws_sdk_secretsmanager::Client;
use aws_types::sdk_config::SdkConfig;
use serde_json::Value;

/// Objects that manages the retrieval of secrets.
#[derive(Clone, Debug)]
pub struct Secrets {
    client: Option<Client>,
}

impl Secrets {
    /// Creates a new instance of secrets manager.
    pub fn new(aws_config: &SdkConfig) -> Self {
        let client = Client::new(aws_config);
        Self {
            client: Some(client),
        }
    }

    /// Creates a new instance with no backing secrets manager.
    pub fn new_test() -> Self {
        Self { client: None }
    }

    /// Fetches a secret from the AWS Secret Manager.
    pub async fn get(&self, name: &str) -> Result<Value> {
        let resp = self
            .client
            .as_ref()
            .context("AWS secrets client is missing")?
            .get_secret_value()
            .secret_id(name)
            .send()
            .await?;

        let secret = resp.secret_string().context("missing secret string")?;
        Ok(serde_json::from_str(secret)?)
    }
}
