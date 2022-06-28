//! Templating engine for cadre templates.

use anyhow::bail;
use anyhow::Result;
use async_recursion::async_recursion;
use aws_types::sdk_config::SdkConfig;
use serde_json::{Map, Value};

use crate::secrets::Secrets;

/// Marker character used for template strings.
pub const TEMPLATE_MARK: &str = "*";

/// A JSON configuration object with templated values.
pub struct Template {
    value: Value,
    secrets: Secrets,
}

impl Template {
    /// Creates new template based on serde JSON Value.
    pub async fn new(aws_config: &SdkConfig, value: Value) -> Result<Self> {
        Ok(Self {
            value,
            secrets: Secrets::new(aws_config).await?,
        })
    }

    /// Parses JSON map based on template requirements.
    pub async fn parse(&self) -> Result<Value> {
        let mut parsed_value = self.value.clone();
        if self.value.is_object() {
            let map = parsed_value.as_object_mut().unwrap();
            for (key, value) in map.iter_mut() {
                evaluate(&self.secrets, key, value).await?;
            }

            remove_template_marks(map).await;
            Ok(parsed_value)
        } else {
            bail!("value must be an object")
        }
    }

    /// Turns this template into a value.
    ///
    /// TODO: Remove the [`Template`] struct; it's entirely unnecessary.
    pub fn into_value(self) -> Value {
        self.value
    }
}

/// Evaluate key-value pair, executing function if required by template mark.
#[async_recursion]
async fn evaluate<'a>(secrets: &Secrets, key: &'a str, value: &mut Value) -> Result<()> {
    // If the value is an array, exit early.
    if value.is_array() {
        bail!("arrays cannot be secret values")
    } else if value.is_object() {
        let map = value.as_object_mut().unwrap();
        for (k, v) in map.iter_mut() {
            evaluate(secrets, k, v).await?;
        }
    } else {
        // Parse templated function and overwrite the value of the key
        // in pointer.
        if key.starts_with(TEMPLATE_MARK) {
            let _value = String::from(value.as_str().unwrap()).to_lowercase();
            let secret_key = value.as_str().unwrap();
            if _value.starts_with("aws(") {
                let pattern = String::from("aws(");
                let secret_name = secret_key.replace(&pattern, "").replace(')', "");
                *value = secrets.get(&secret_name).await?;
            } else {
            };
        };
    };

    Ok(())
}

/// Remove the template mark from keys.
pub async fn remove_template_marks(map: &mut Map<String, Value>) {
    *map = std::mem::take(map)
        .into_iter()
        .map(|(k, v)| (k.replace(TEMPLATE_MARK, ""), v))
        .collect();
}
