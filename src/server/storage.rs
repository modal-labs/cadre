//! Pluggable storage persistence backend for templates.

use std::collections::HashMap;
use std::path::PathBuf;
use std::str;

use anyhow::{Context, Result};
use parking_lot::Mutex;
use serde_json::Value;
use tokio::fs;
use tracing::info;

/// Storage backend specification.
#[derive(Debug)]
pub enum Storage {
    /// Persist templates to an S3 bucket.
    S3(aws_sdk_s3::Client, String),

    /// Persist templates to the local file system.
    LocalFS(PathBuf),

    /// Only store data in-memory.
    Memory(Mutex<HashMap<String, Value>>),
}

impl Storage {
    /// Retrieve a value from storage.
    #[tracing::instrument]
    pub(crate) async fn get(&self, env: &str) -> Result<Value> {
        info!("reading template");
        match self {
            Storage::S3(s3, bucket) => {
                let key = format!("{env}.json");
                let resp = s3.get_object().bucket(bucket).key(key).send().await?;
                let data = resp.body.collect().await?;
                let bytes = data.into_bytes();
                Ok(serde_json::from_str(str::from_utf8(&bytes)?)?)
            }
            Storage::LocalFS(path) => {
                let path = path.join(format!("{env}.json"));
                let data = fs::read(&path).await?;
                Ok(serde_json::from_slice(&data)?)
            }
            Storage::Memory(map) => Ok(map.lock().get(env).context("missing template")?.clone()),
        }
    }

    /// Set a value in storage.
    #[tracing::instrument]
    pub(crate) async fn set(&self, env: &str, value: &Value) -> Result<()> {
        info!("writing template");
        match self {
            Storage::S3(s3, bucket) => {
                let key = format!("{env}.json");
                let content = serde_json::to_vec(value)?.into();
                s3.put_object()
                    .bucket(bucket)
                    .key(key)
                    .body(content)
                    .send()
                    .await?;
            }
            Storage::LocalFS(path) => {
                let path = path.join(format!("{env}.json"));
                let content = serde_json::to_vec(value)?;
                fs::write(&path, &content).await?;
            }
            Storage::Memory(map) => {
                map.lock().insert(env.into(), value.clone());
            }
        }
        Ok(())
    }

    /// List all of the templates in storage.
    #[tracing::instrument]
    pub(crate) async fn list(&self) -> Result<Vec<String>> {
        info!("listing templates");
        match self {
            Storage::S3(s3, bucket) => {
                let objects = s3.list_objects_v2().bucket(bucket).send().await?;
                Ok(objects
                    .contents()
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|object| Some(object.key()?.strip_suffix(".json")?.to_owned()))
                    .collect())
            }
            Storage::LocalFS(path) => {
                let mut dir = fs::read_dir(&path).await?;
                let mut results = Vec::new();
                while let Some(entry) = dir.next_entry().await? {
                    if let Some(name) = entry.file_name().to_str() {
                        if let Some(env) = name.strip_suffix(".json") {
                            results.push(env.to_owned());
                        }
                    }
                }
                Ok(results)
            }
            Storage::Memory(map) => Ok(map.lock().keys().cloned().collect()),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serde_json::json;

    use super::Storage;

    #[tokio::test]
    async fn memory_operations() -> Result<()> {
        let storage = Storage::Memory(Default::default());

        assert!(storage.get("hello").await.is_err());
        assert!(storage.list().await?.is_empty());

        storage.set("hello", &json!("world")).await?;
        assert_eq!(storage.get("hello").await?, json!("world"));
        assert_eq!(storage.list().await?, vec![String::from("hello")]);

        Ok(())
    }
}
