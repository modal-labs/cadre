//! Templating engine for cadre templates.

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use serde_json::Value;

use crate::secrets::Secrets;

/// Marker character used for template strings.
pub const TEMPLATE_MARK: &str = "*";

/// Populate a JSON value with the results of templated strings.
pub async fn populate_template(value: &mut Value, secrets: &Secrets) -> Result<()> {
    let mut stack = vec![value];
    while let Some(value) = stack.pop() {
        if let Some(map) = value.as_object_mut() {
            for (key, value) in std::mem::take(map) {
                if let Some(key_raw) = key.strip_prefix(TEMPLATE_MARK) {
                    let value = value
                        .as_str()
                        .with_context(|| format!("templated key {key:?} is of non-string type"))?;
                    map.insert(
                        key_raw.into(),
                        resolve_templated_value(value, secrets).await?,
                    );
                } else {
                    map.insert(key, value);
                }
            }
            stack.extend(map.values_mut());
        }
    }
    Ok(())
}

/// Resolve a single value that uses the template language.
async fn resolve_templated_value(value: &str, secrets: &Secrets) -> Result<Value> {
    if let Some(name) = value.strip_prefix("aws:") {
        secrets.get(name).await
    } else {
        bail!("unresolved template syntax: {value:?}");
    }
}

/// Merge two raw template objects, remaining aware of unpopulated values.
pub fn merge_templates(dest: &mut Value, src: &Value) {
    match (dest, src) {
        (&mut Value::Object(ref mut dest), &Value::Object(ref src)) => {
            for (key, value) in src {
                // Remove template mark, otherwise we won't be able to merge correctly and
                // both keys--templated and not--templated—will exist in the resulting JSON.
                if let Some(key_raw) = key.strip_prefix(TEMPLATE_MARK) {
                    dest.remove(key_raw);
                }
                merge_templates(dest.entry(key.clone()).or_insert(Value::Null), value);
            }
        }
        (dest, src) => {
            *dest = src.clone();
        }
    }
}
