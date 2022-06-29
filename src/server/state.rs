//! Server state object managing all operations on cadre configuration.

use std::str;
use std::sync::Arc;

use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client;
use aws_types::sdk_config::SdkConfig;
use serde_json::Value;

use super::resolver::{AwsSecrets, ResolverChain};
use super::storage::Storage;
use super::template::{merge_templates, populate_template};

/// Creates an AWS SDK default config object.
pub async fn default_aws_config() -> SdkConfig {
    let region_provider = RegionProviderChain::default_provider();
    aws_config::from_env().region(region_provider).load().await
}

/// Object that manages server state, including storage and templating.
#[derive(Clone)]
pub struct State {
    chain: Arc<ResolverChain>,
    storage: Arc<Storage>,
    default_template: Option<String>,
}

impl State {
    /// Create a new state object.
    pub fn new(chain: ResolverChain, storage: Storage, default_template: Option<&str>) -> Self {
        Self {
            chain: Arc::new(chain),
            storage: Arc::new(storage),
            default_template: default_template.map(String::from),
        }
    }

    /// Initialize the default state for the server.
    pub async fn from_env(bucket: &str, default_template: Option<&str>) -> Self {
        let config = default_aws_config().await;
        let mut chain = ResolverChain::new();
        chain.add(AwsSecrets::new(&config));
        let storage = Storage::S3(Client::new(&config), bucket.into());
        Self::new(chain, storage, default_template)
    }

    /// Read a configuration template from S3.
    pub async fn read_template(&self, env: &str) -> Result<Value> {
        self.storage.get(env).await
    }

    /// Atomically persist a configuration template to S3.
    pub async fn write_template(&self, env: &str, template: &Value) -> Result<()> {
        self.storage.set(env, template).await
    }

    /// Read a configuration template from S3 and populate templated values.
    ///
    /// This configuration will first be merged with the default template as
    /// well, if it is provided.
    pub async fn load_config(&self, env: &str) -> Result<Value> {
        let mut template = self.read_template(env).await?;
        if let Some(default_env) = &self.default_template {
            if env != default_env {
                let default_template = self.read_template(default_env).await?;
                merge_templates(&mut template, &default_template)
            }
        }
        populate_template(&mut template, &self.chain).await?;
        Ok(template)
    }

    /// Return a list of available configuration templates from S3.
    pub async fn list_available_configs(&self) -> Result<Vec<String>> {
        self.storage.list().await
    }
}
