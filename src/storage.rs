//! Persistent, durable storage for cadre configuration.
//!
//! This stores JSON templates in a S3 bucket.

use std::str;

use anyhow::{Context, Result};
use async_recursion::async_recursion;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client;
use aws_types::sdk_config::SdkConfig;
use serde_json::Value;
use tracing::info;

use crate::template::{Template, TEMPLATE_MARK};

/// Creates an AWS SDK default config object.
pub async fn default_aws_config() -> Result<SdkConfig> {
    let region_provider = RegionProviderChain::default_provider();
    Ok(aws_config::from_env().region(region_provider).load().await)
}

/// Object that manages storage persistence.
#[derive(Clone, Debug)]
pub struct Storage {
    client: Client,
    bucket: String,
    default_template: Option<String>,
    aws_config: SdkConfig,
}

impl Storage {
    /// Create a new storage object.
    pub async fn new(bucket: &str, default_template: Option<&str>) -> Result<Self> {
        let config = default_aws_config().await?;
        Ok(Self {
            client: Client::new(&config),
            bucket: bucket.into(),
            default_template: default_template.map(String::from),
            aws_config: config,
        })
    }

    async fn fetch_object(&self, environment: &str) -> Result<Value> {
        info!(%environment, "reading object");

        let key = format!("{environment}.json");
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        let data = resp.body.collect().await?;
        let bytes = data.into_bytes();
        Ok(serde_json::from_str(str::from_utf8(&bytes)?)?)
    }

    /// Merges requested template with default if required.
    async fn merge_with_default_template(
        &self,
        templated_json: Template,
        environment: &str,
    ) -> Result<Value> {
        Ok(match self.default_template.as_deref() {
            // The environment called is the same as the default template.
            Some(env) if env == environment => templated_json.into_value(),

            // Merge new template with the default template, overriding the default
            // template keys.
            Some(env) => {
                let mut d_template = self.read_template(env).await?;
                merge_values(&mut d_template, &templated_json.into_value());
                d_template
            }

            None => templated_json.into_value(),
        })
    }

    /// Read a configuration template from S3.
    #[async_recursion]
    pub async fn read_template(&self, environment: &str) -> Result<Value> {
        let json = self.fetch_object(environment).await?;
        let templated_json = Template::new(&self.aws_config, json).await?;
        let value = self
            .merge_with_default_template(templated_json, environment)
            .await?;

        Ok(value)
    }

    /// Read a configuration from S3 with populated template values.
    #[async_recursion]
    pub async fn read_config(&self, environment: &str) -> Result<Value> {
        let merged_value = self.read_template(environment).await?;

        // Parse the resulting templatet into values.
        let merged_template = Template::new(&self.aws_config, merged_value).await?;
        Ok(merged_template.parse().await?)
    }

    /// Atomically persist a JSON configuration object into storage.
    pub async fn write(&self, environment: &str, value: &Value) -> Result<()> {
        info!(%environment, "writing configuration");
        let key = format!("{environment}.json");
        let content = serde_json::to_vec(value)?.into();

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(content)
            .send()
            .await?;

        Ok(())
    }

    /// Returns list of available config files from S3.
    pub async fn list_available_configs(&self) -> Result<Value> {
        let objects = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .send()
            .await?;

        let mut configs = Vec::new();
        for obj in objects.contents().unwrap_or_default() {
            // Only return json files to users.
            let object_name = obj.key().context("S3 object is missing key")?;
            if let Some(stripped) = object_name.strip_suffix(".json") {
                configs.push(stripped)
            }
        }
        Ok(configs.into())
    }
}

/// Merge two [`serde_json::Value`] objects.
///
/// Reference: https://github.com/serde-rs/json/issues/377#issuecomment-341490464
fn merge_values(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                // Remove template mark, otherwise we won't be able to merge correctly and
                // both keys--templated and not-templated--will exist in the resulting
                // json.
                let k_no_mark = k.strip_prefix(TEMPLATE_MARK).unwrap_or(k);
                a.remove(k_no_mark);
                merge_values(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}
