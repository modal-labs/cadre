//! Persistent, durable storage for cadre configuration.
//!
//! This stores JSON templates in a S3 bucket.
use std::str::from_utf8;

use anyhow::Result;
use async_recursion::async_recursion;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client;
use aws_types::sdk_config::SdkConfig;
use serde_json::Value;
use tracing::info;

use crate::template::Template;

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
    pub async fn new(bucket: String, default_template: Option<String>) -> Result<Self> {
        let config = default_aws_config().await?;
        Ok(Self {
            client: Client::new(&config),
            bucket,
            default_template,
            aws_config: config,
        })
    }

    async fn fetch_object(&self, environment: &str) -> Result<Value> {
        info!(%environment, "reading object");

        let key = format!("{environment}.json");
        let resp = self
            .client
            .get_object()
            .bucket(self.bucket.clone())
            .key(key)
            .send()
            .await?;

        let data = resp.body.collect().await;
        let bytes = data.unwrap().into_bytes();
        let json = serde_json::from_str(from_utf8(&bytes)?)?;

        Ok(json)
    }

    /// Merges requested template with default if required.
    async fn merge_with_default_template(
        &self,
        templated_json: Template,
        environment: &str,
    ) -> Result<Value> {
        let value = match self.default_template.clone() {
            // The environment called is the same as the default template.
            _ if self.default_template == Option::from(environment.to_string()) => {
                templated_json.value
            }

            // Merge new template with the default template, overriding the default
            // template keys.
            Some(v) => {
                let mut d_template = self.read_template(&v).await?;
                merge_values(
                    &mut d_template,
                    &templated_json.value,
                    &templated_json.template_mark,
                );
                d_template
            }
            None => templated_json.value,
        };

        Ok(value)
    }

    /// Get config template from S3.
    #[async_recursion]
    pub async fn read_template(&self, environment: &str) -> Result<Value> {
        let json = self.fetch_object(environment).await?;
        let templated_json = Template::new(&self.aws_config, json).await?;
        let value = self
            .merge_with_default_template(templated_json, environment)
            .await?;

        Ok(value)
    }

    /// Get and parse config from S3.
    #[async_recursion]
    pub async fn read_parsed_template(&self, environment: &str) -> Result<Value> {
        info!(%environment, "reading template");

        // Get object from S3 and merge with defaults.
        let json = self.fetch_object(environment).await?;
        let templated_json = Template::new(&self.aws_config, json).await?;
        let merged_value = self
            .merge_with_default_template(templated_json, environment)
            .await?;

        // Parse the resulting templatet into values.
        let mut merged_template = Template::new(&self.aws_config, merged_value).await?;
        let parsed_value = merged_template.parse().await?;

        Ok(parsed_value)
    }

    /// Atomically persist a JSON configuration object into storage.
    pub async fn write(&self, environment: &String, value: &Value) -> Result<()> {
        info!(%environment, "writing configuration");
        let key = format!("{environment}.json");
        let content = serde_json::to_vec(value)?.into();

        self.client
            .put_object()
            .bucket(self.bucket.clone())
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
            // only return json files to users; remove extension for easy subsequent
            // operations
            let object_name = obj.key().unwrap();
            if object_name.ends_with("json") {
                configs.push(object_name.replace(".json", ""))
            } else {
            }
        }

        let value = Value::from(configs);
        Ok(serde_json::from_value(value)?)
    }
}

// Merges two serde_json::Value objects.
// Reference: https://github.com/serde-rs/json/issues/377#issuecomment-341490464
fn merge_values(a: &mut Value, b: &Value, template_mark: &str) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                // Remove template mark, otherwise we won't be able to merge correctly and
                // both keys--templated and not-templated--will exist in the resulting
                // json.
                let mut k_mut = k.clone();
                if k_mut.starts_with(template_mark) {
                    k_mut = k_mut.replace(template_mark, "");
                };
                a.remove(&k_mut);
                merge_values(a.entry(k.clone()).or_insert(Value::Null), v, template_mark);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}
