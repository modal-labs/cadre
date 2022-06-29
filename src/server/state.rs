//! Server state object managing all operations on cadre configuration.

use std::str;
use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;

use super::resolver::ResolverChain;
use super::storage::Storage;
use super::template::{merge_templates, populate_template};

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
