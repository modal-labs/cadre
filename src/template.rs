//! Templating engine for cadre templates.

use anyhow::bail;
use anyhow::Result;
use async_recursion::async_recursion;
use aws_types::sdk_config::SdkConfig;
use serde_json::{Map, Value};

use crate::secrets::Secrets;

/// Templated JSON object.
pub struct Template {
    pub value: Value,
    secrets: Secrets,
    pub template_mark: String,
}

impl Template {
    /// Creates new template based on serde JSON Value.
    pub async fn new(aws_config: &SdkConfig, value: Value) -> Result<Self> {
        Ok(Self {
            value,
            secrets: Secrets::new(aws_config).await?,
            template_mark: String::from("*"), // Keys that start with this character.
        })
    }

    /// Parses JSON map based on template requirements.
    pub async fn parse(&mut self) -> Result<Value> {
        let mut parsed_value = self.value.clone();
        if self.value.is_object() {
            let map = parsed_value.as_object_mut().unwrap();
            for (key, value) in map.iter_mut() {
                evaluate(&self.secrets, key, value, &self.template_mark).await?;
            }

            remove_template_marks(&self.template_mark, parsed_value.as_object_mut().unwrap()).await;
            Ok(parsed_value)
        } else {
            bail!("value must be an object")
        }
    }
}

/// Evaluate key-value pair, executing function if required by template mark.
#[async_recursion]
async fn evaluate(
    secrets: &Secrets,
    key: &'life1 str,
    value: &mut Value,
    template_mark: &String,
) -> Result<()> {
    // If the value is an array, exit early.
    if value.is_array() {
        bail!("arrays cannot be secret values")
    } else if value.is_object() {
        let map = value.as_object_mut().unwrap();
        for (k, v) in map.iter_mut() {
            evaluate(secrets, k, v, template_mark).await?;
        }
    } else {
        // Parse templated function and overwrite the value of the key
        // in pointer.
        if key.starts_with(template_mark) {
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
pub async fn remove_template_marks(mark: &String, map: &mut Map<String, Value>) {
    *map = std::mem::take(map)
        .into_iter()
        .map(|(k, v)| (k.replace(mark, ""), v))
        .collect();
}
