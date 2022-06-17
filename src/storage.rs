//! Persistent, durable storage for cadre configuration.
//!
//! This stores a single file in `~/.cadre/config.json`, which is atomically
//! updated through file system move operations.
use std::{io::ErrorKind, ops::Deref, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use serde_json::{json, Value};
use tokio::{fs, sync::RwLock, task};

use s3::bucket::Bucket;
use s3::creds::Credentials;

/// Object that manages storage persistence.
#[derive(Clone, Debug)]
pub struct Storage {
    bucket: Bucket,
}

impl Storage {
    /// Create a new storage object.
    pub async fn new(bucket_name: String) -> Result<Self> {
        let region = String::from("us-east-1").parse()?;
        let credentials = Credentials::default()?;

        Ok(Self {
            bucket: Bucket::new(&bucket_name, region, credentials)?,
        })
    }

    /// Get current config template from S3.
    pub async fn read(&self, path: &str) -> Result<Value> {
        let (result, _) = self.bucket.get_object(path).await?;
        let value = Value::from(result);
        Ok(serde_json::from_value(value)?)
    }

    /// Returns list of available config files from S3.
    // pub async fn available_configs(&self) -> Result<()> {
    //     let results = self.bucket.list("".to_string(), None).await?;
    //     Ok(())
    // }

    /// Atomically persist a JSON configuration object into storage.
    pub async fn write(&self, value: &Value) -> Result<()> {
        let content = serde_json::to_vec(value)?;
        self.bucket.put_object("config.json", &content).await?;

        // all good
        Ok(())
    }
}
