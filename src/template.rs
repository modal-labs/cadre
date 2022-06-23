//! Templating engine for cadre templates.

use anyhow::bail;
use anyhow::Result;
use async_recursion::async_recursion;
use serde_json::{Map, Value};

use crate::secrets::Secrets;

/// Templated JSON object.
pub struct Template {
    pub value: Value,
    secrets: Secrets,
    template_mark: String,
}

impl Template {
    /// Creates new template based on serde JSON Value.
    pub async fn new(value: Value) -> Result<Self> {
        Ok(Self {
            value,
            secrets: Secrets::new(String::from("us-east-1")).await?,
            template_mark: String::from("*"), // Keys that start with this character.
        })
    }

    /// Parses JSON map based on template requirements.
    pub async fn parse(&mut self) -> Result<Value> {
        let mut parsed_value = self.value.clone();
        if self.value.is_array() {
            bail!("based config objects cannot be arrays")
        } else {
            let map = parsed_value.as_object_mut().unwrap();
            for (key, value) in map.iter_mut() {
                evaluate(&self.secrets, key, value, &self.template_mark).await?;
            }

            remove_template_marks(&self.template_mark, parsed_value.as_object_mut().unwrap()).await;
            Ok(parsed_value)
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
        // When value is an object,
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
                let secret_name = _extract_function_value(pattern, secret_key);
                *value = secrets.get(&secret_name).await?;
            } else if _value.starts_with("aws_json(") {
                let pattern = String::from("aws_json(");
                let secret_name = _extract_function_value(pattern, secret_key);
                *value = secrets.get_as_map(&secret_name).await?;
            } else {
            };
        };
    };

    Ok(())
}

/// Replace map in memory with equivalent map but with replaced templated keys.
// #[async_recursion]
// async fn remove_template_marks(template_mark: &String, value: &mut Value) -> Result<()> {
//     let map = value.as_object_mut().unwrap();
//     let mut _map = map.clone();
//     for (k, v) in map.iter_mut() {
//         if v.is_array() {
//         } else if v.is_object() {
//             remove_template_marks(template_mark, v).await?;
//         } else {
//             if k.starts_with(template_mark) {
//                 let mut _k = k.clone();
//                 _k = k.replace(template_mark, "");
//                 _map[&_k] = v.clone();
//             }
//         }
//     }
//     // TOOD: from Map to serde_json::Value
//     *value = Value::Object(_map.clone());

//     Ok(())
// }

fn _extract_function_value(pattern: String, value: &str) -> String {
    value.replace(&pattern, "").replace(')', "")
}

async fn remove_template_marks(template_mark: &String, map: &mut Map<String, Value>) {
    *map = std::mem::take(map)
        .into_iter()
        .map(|(k, v)| (k.replace(template_mark, ""), v))
        .collect();
}
