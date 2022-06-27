//! Persistent, durable storage for cadre configuration.
//!
//! This stores JSON templates in a S3 bucket.
use std::str::from_utf8;

use anyhow::Result;
use async_recursion::async_recursion;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::Client;
use aws_types::sdk_config::SdkConfig;
use bytes::Bytes;
use serde_json::Value;

use crate::template::Template;

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
        let region_provider = RegionProviderChain::default_provider();
        let config = aws_config::from_env().region(region_provider).load().await;

        Ok(Self {
            client: Client::new(&config),
            bucket,
            default_template,
            aws_config: config,
        })
    }

    async fn fetch_object(&self, environment: &str) -> Result<Value> {
        println!(" => read environment: '{}'", environment);

        let key = add_json_extension(environment);
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
    async fn merge_defaults(&self, templated_json: Template, environment: &str) -> Result<Value> {
        let value = match self.default_template.clone() {
            _ if self.default_template == Option::from(environment.to_string()) => {
                templated_json.value
            }
            Some(v) => {
                let mut d_template = self.read_template(&v).await?;
                merge_values(&mut d_template, &templated_json.value);
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
        let value = self.merge_defaults(templated_json, environment).await?;

        Ok(value)
    }

    /// Get and parse config from S3.
    pub async fn read_parsed_template(&self, environment: &str) -> Result<Value> {
        println!(" => read environment: '{}'", environment);

        // Get object from S3 and merge with defaults.
        let json = self.fetch_object(environment).await?;
        let templated_json = Template::new(&self.aws_config, json).await?;
        let merged_value = self.merge_defaults(templated_json, environment).await?;

        // Parse the resulting templatet into values.
        let mut merged_template = Template::new(&self.aws_config, merged_value).await?;
        let parsed_value = merged_template.parse().await?;

        Ok(parsed_value)
    }

    /// Atomically persist a JSON configuration object into storage.
    pub async fn write(&self, environment: &String, value: &Value) -> Result<()> {
        println!(" => writing environment: '{}'", environment);
        let key = add_json_extension(environment);
        let bytes = Bytes::copy_from_slice(&serde_json::to_vec(value)?);
        let content = ByteStream::from(bytes);

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

fn add_json_extension(environment: &str) -> String {
    format!("{}.json", environment)
}

// Merges two serde_json::Value objects.
// Reference: https://github.com/serde-rs/json/issues/377#issuecomment-341490464
fn merge_values(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge_values(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}
