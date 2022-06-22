//! Templating engine for cadre templates.

use anyhow::bail;
use anyhow::Result;
use async_recursion::async_recursion;
use serde_json::Value;

use crate::secrets::Secrets;

pub struct Template {
    pub value: Value,
    secrets: Secrets,
    template_mark: String,
}

impl Template {
    /// Creates new template based on serde JSON Value.
    pub async fn new(value: Value) -> Result<Self> {
        Ok(Self {
            value: value,
            secrets: Secrets::new(String::from("us-east-1")).await?,
            template_mark: String::from("*"), // Keys that start with this character.
        })
    }

    /// Parses JSON map based on template requirements.
    pub async fn parse(&mut self) -> Result<()> {
        if self.value.is_array() {
            bail!("based config objects cannot be arrays")
        } else {
            let map = self.value.as_object_mut().unwrap();
            for (key, value) in map.iter_mut() {
                evaluate(&self.secrets, key, value, &self.template_mark).await?;
            }
            Ok(())
        }
    }
}

/// Evaluate key-value pair, executing function if required by template mark.
#[async_recursion]
async fn evaluate(
    secrets: &Secrets,
    key: &String,
    value: &mut Value,
    template_mark: &String,
) -> Result<()> {
    // If the value is an array, exit early.
    if value.is_array() {
        bail!("arrays cannot be secret values")
    } else if value.is_object() {
        // When value is an object,
        let map = value.as_object_mut().unwrap();
        for (k, v) in map.iter_mut() {
            evaluate(secrets, k, v, template_mark).await?;
        }
    } else {
        if key.starts_with(template_mark) {
            let _value = String::from(value.as_str().unwrap()).to_lowercase();
            let secret_key = value.as_str().unwrap();
            if _value.starts_with("aws(") {
                let pattern = String::from("aws(");
                let secret_name = extract_function_value(pattern, secret_key);
                *value = secrets.get(&secret_name).await?;
            } else if _value.starts_with("aws_json(") {
                let pattern = String::from("aws_json(");
                let secret_name = extract_function_value(pattern, secret_key);
                *value = secrets.get_as_map(&secret_name).await?;
            } else {
            };
        };
    };
    Ok(())
}

fn extract_function_value(pattern: String, value: &str) -> String {
    value.replace(&pattern, "").replace(")", "")
}
