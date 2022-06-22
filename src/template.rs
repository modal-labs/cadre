//! Templating engine for cadre templates.

use anyhow::bail;
use anyhow::Result;
use async_recursion::async_recursion;
use serde_json::Value;

pub struct Template {
    pub value: Value,
    template_mark: String,
}

impl Template {
    /// Creates new template based on serde JSON Value.
    pub async fn new(value: Value) -> Result<Self> {
        Ok(Self {
            value: value,
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
                evaluate(key, value, &self.template_mark).await?;
            }
            Ok(())
        }
    }
}

/// Evaluate key-value pair, executing function if required by template mark.
#[async_recursion]
async fn evaluate(key: &String, value: &mut Value, template_mark: &String) -> Result<()> {
    // If the value is an array, exit early.
    if value.is_array() {
        Err(())
    } else if value.is_object() {
        // When value is an object,
        let map = value.as_object_mut().unwrap();
        for (k, v) in map.iter_mut() {
            evaluate(k, v, template_mark).await?;
        }
        Ok(())
    } else {
        if key.starts_with(template_mark) {
            get_aws_secret(value.as_str())?;
        };
        Ok(())
    };
    Ok(())
}

/// Gets secret from the AWS secret manager.
fn get_aws_secret(secret_name: Option<&str>) -> Result<()> {
    match secret_name {
        Some(secret_name) => {
            println!("value for key: {}", secret_name);
        }
        None => {
            println!("value is none");
        }
    }

    Ok(())
}
