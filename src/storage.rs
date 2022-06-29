//! Persistent, durable storage for cadre configuration.
//!
//! This stores JSON templates in a S3 bucket.

use std::str;

use anyhow::{Context, Result};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client;
use aws_types::sdk_config::SdkConfig;
use serde_json::Value;
use tracing::info;

use crate::secrets::Secrets;
use crate::template::{merge_templates, populate_template};

/// Creates an AWS SDK default config object.
pub async fn default_aws_config() -> Result<SdkConfig> {
    let region_provider = RegionProviderChain::default_provider();
    Ok(aws_config::from_env().region(region_provider).load().await)
}

/// Object that manages storage persistence.
#[derive(Clone, Debug)]
pub struct Storage {
    s3: Client,
    secrets: Secrets,
    bucket: String,
    default_template: Option<String>,
}

impl Storage {
    /// Create a new storage object.
    pub async fn new(bucket: &str, default_template: Option<&str>) -> Result<Self> {
        let config = default_aws_config().await?;
        Ok(Self {
            s3: Client::new(&config),
            secrets: Secrets::new(&config),
            bucket: bucket.into(),
            default_template: default_template.map(String::from),
        })
    }

    /// Read a configuration template from S3.
    pub async fn read_template(&self, env: &str) -> Result<Value> {
        info!(%env, "reading config template");

        let key = format!("{env}.json");
        let resp = self
            .s3
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        let data = resp.body.collect().await?;
        let bytes = data.into_bytes();
        Ok(serde_json::from_str(str::from_utf8(&bytes)?)?)
    }

    /// Atomically persist a configuration template to S3.
    pub async fn write_template(&self, env: &str, template: &Value) -> Result<()> {
        info!(%env, "writing config template");
        let key = format!("{env}.json");
        let content = serde_json::to_vec(template)?.into();

        self.s3
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(content)
            .send()
            .await?;

        Ok(())
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
        populate_template(&mut template, &self.secrets).await?;
        Ok(template)
    }

    /// Return a list of available configuration templates from S3.
    pub async fn list_available_configs(&self) -> Result<Value> {
        let objects = self
            .s3
            .list_objects_v2()
            .bucket(&self.bucket)
            .send()
            .await?;

        let mut configs = Vec::new();
        for obj in objects.contents().unwrap_or_default() {
            // Only return json files to users.
            let object_name = obj.key().context("S3 object is missing key")?;
            if let Some(stripped) = object_name.strip_suffix(".json") {
                configs.push(stripped);
            }
        }
        Ok(configs.into())
    }
}
