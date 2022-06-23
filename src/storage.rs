//! Persistent, durable storage for cadre configuration.
//!
//! This stores JSON templates in a S3 bucket.
use anyhow::Result;
use bytes::Bytes;
use serde_json::Value;

use std::str::from_utf8;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::Client;

use crate::template::Template;

/// Object that manages storage persistence.
#[derive(Clone, Debug)]
pub struct Storage {
    client: Client,
    bucket: String,
}

impl Storage {
    /// Create a new storage object.
    pub async fn new(bucket: String) -> Result<Self> {
        // TODO (luiscape): parametrize region as part of CLI
        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let config = aws_config::from_env().region(region_provider).load().await;

        Ok(Self {
            client: Client::new(&config),
            bucket,
        })
    }

    /// Get config template from S3.
    pub async fn read_template(&self, environment: &str) -> Result<Value> {
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

        let templated_json = Template::new(json).await?;
        Ok(templated_json.value)
    }

    /// Get and parse config from S3.
    pub async fn read_parsed_template(&self, environment: &str) -> Result<Value> {
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

        let mut templated_json = Template::new(json).await?;
        let parsed_value = templated_json.parse().await?;

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
            configs.push(obj.key().unwrap());
        }

        let value = Value::from(configs);
        Ok(serde_json::from_value(value)?)
    }
}

fn add_json_extension(environment: &str) -> String {
    format!("{}.json", environment)
}
