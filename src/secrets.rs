//! Interface for secret retrieval from the AWS Secret Manager service.;

use anyhow::Result;
use aws_sdk_secretsmanager::Client;
use aws_types::sdk_config::SdkConfig;
use serde_json::{from_str, Value};

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
        let secret = Value::from(resp.secret_string().unwrap_or(""));
        let value = from_str(secret.as_str().unwrap())?;

        Ok(value)
    }
}
