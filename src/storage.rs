//! Persistent, durable storage for cadre configuration.
//!
//! This stores a single file in `~/.cadre/config.json`, which is atomically
//! updated through file system move operations.

use std::{io::ErrorKind, ops::Deref, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use serde_json::{json, Value};
use tempfile::NamedTempFile;
use tokio::{fs, sync::RwLock, task};

/// Object that manages storage persistence.
#[derive(Clone, Debug)]
pub struct Storage {
    data: Arc<RwLock<Value>>,
    path: Arc<PathBuf>,
}

impl Storage {
    /// Create a new storage object.
    pub async fn new() -> Result<Self> {
        let dir = home::home_dir().context("no home dir")?.join(".cadre");
        fs::create_dir_all(&dir).await?;

        let path = dir.join("config.json");
        let value = match fs::read(&path).await {
            Ok(data) => serde_json::from_slice(&data)?,
            Err(err) if err.kind() == ErrorKind::NotFound => json!({}),
            Err(err) => return Err(err.into()),
        };

        Ok(Self {
            data: Arc::new(RwLock::new(value)),
            path: Arc::new(path),
        })
    }

    /// Get the current value of the configuration.
    pub async fn read(&self) -> impl Deref<Target = Value> + '_ {
        self.data.read().await
    }

    /// Atomically persist a JSON configuration object into storage.
    pub async fn write(&self, value: &Value) -> Result<()> {
        // create new temp file
        let file = task::spawn_blocking(NamedTempFile::new).await??;

        // write values into temp file
        fs::write(file.path(), serde_json::to_vec(value)?).await?;

        // just one write at a time
        let mut data = self.data.write().await;
        let path = Arc::clone(&self.path);

        // move temp file to target location
        task::spawn_blocking(move || file.persist(&*path)).await??;
        *data = value.clone();

        // all good
        Ok(())
    }
}
