//! Interfaces for populating special values in config templates.

use std::collections::HashMap;

use anyhow::{Context, Result};
use async_trait::async_trait;
use aws_sdk_secretsmanager::Client;
use aws_types::sdk_config::SdkConfig;
use serde_json::Value;

use super::cache::TimedCache;

/// Collection of resolvers for populating values in templates.
#[derive(Default)]
pub struct ResolverChain {
    map: HashMap<&'static str, Box<dyn Resolver>>,
}

impl ResolverChain {
    /// Create an empty resolver chain.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a new resolver to the store, if it does not conflict in prefix.
    pub fn add(&mut self, resolver: impl Resolver + 'static) -> bool {
        let prefix = resolver.prefix();
        if self.map.contains_key(prefix) {
            return false;
        }
        self.map.insert(prefix, Box::new(resolver));
        true
    }

    /// Resolve a templated value, including its prefix.
    pub async fn resolve(&self, value: &str) -> Result<Value> {
        let (prefix, name) = value
            .split_once(':')
            .context("templated value is missing delimiter character ':'")?;
        let resolver = self.map.get(prefix).with_context(|| {
            format!(
                "could not find prefix {prefix} in the list of resolvers: {:?}",
                self.map.keys().collect::<Vec<_>>()
            )
        })?;
        resolver.resolve(name).await
    }
}

/// Trait for resolving special keys in templates.
#[async_trait]
pub trait Resolver: Send + Sync {
    /// The prefix of this resolver, as used in templates.
    fn prefix(&self) -> &'static str;

    /// Fetches a secret by value.
    async fn resolve(&self, name: &str) -> Result<Value>;
}

/// Client for retrieving secrets from AWS Secrets Manager.
pub struct AwsSecrets {
    client: Client,
    cache: TimedCache<String, Value>,
}

impl AwsSecrets {
    /// Creates a new instance of secrets manager.
    pub fn new(aws_config: &SdkConfig) -> Self {
        let client = Client::new(aws_config);
        Self {
            client,
            cache: TimedCache::new(45.0, 60.0),
        }
    }
}

#[async_trait]
impl Resolver for AwsSecrets {
    fn prefix(&self) -> &'static str {
        "aws"
    }

    async fn resolve(&self, name: &str) -> Result<Value> {
        if let Some(value) = self.cache.get(name) {
            return Ok(value);
        }

        let resp = self
            .client
            .get_secret_value()
            .secret_id(name)
            .send()
            .await?;

        let secret = resp.secret_string().context("missing secret string")?;
        let value: Value = serde_json::from_str(secret)?;
        self.cache.insert(name.into(), value.clone());
        Ok(value)
    }
}

/// A resolver that simply echos the input as JSON, used for testing.
#[doc(hidden)]
pub struct EchoJson;

#[doc(hidden)]
#[async_trait]
impl Resolver for EchoJson {
    fn prefix(&self) -> &'static str {
        "echo"
    }

    async fn resolve(&self, name: &str) -> Result<Value> {
        Ok(serde_json::from_str(name)?)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serde_json::json;

    use super::{EchoJson, ResolverChain};

    #[tokio::test]
    async fn empty_resolver() {
        let chain = ResolverChain::new();
        assert!(chain.resolve("hello:world").await.is_err());
    }

    #[tokio::test]
    async fn echo_resolver() -> Result<()> {
        let mut chain = ResolverChain::new();
        assert!(chain.add(EchoJson));
        assert_eq!(chain.resolve("echo:\"world\"").await?, json!("world"));
        assert!(chain.resolve("hello:world").await.is_err());

        assert!(!chain.add(EchoJson));
        Ok(())
    }
}
