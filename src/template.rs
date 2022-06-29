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
///
/// This does not overwrite data. If a property is present in both the
/// destination template and the source template, then the destination
/// template's value will be preserved.
pub fn merge_templates(dest: &mut Value, src: &Value) {
    if let (Value::Object(dest), Value::Object(src)) = (dest, src) {
        for (key, value) in src {
            let key_raw = key.strip_prefix(TEMPLATE_MARK).unwrap_or(key);
            if !dest.contains_key(key_raw)
                && !dest.contains_key(&format!("{TEMPLATE_MARK}{key_raw}"))
            {
                // Total replacement: dest does not contain the key.
                dest.insert(key.clone(), value.clone());
            } else if key == key_raw && dest.contains_key(key) {
                // Partial replacement: both contain raw non-templated key,
                // so we can merge their objects' subkeys.
                merge_templates(dest.get_mut(key).unwrap(), value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::merge_templates;

    #[test]
    fn merge_templates_basic() {
        let mut dest = json!({ "a": "b" });
        let src = json!({ "c": "d" });
        merge_templates(&mut dest, &src);
        assert_eq!(dest, json!({ "a": "b", "c": "d" }));
    }

    #[test]
    fn merge_templates_replacement() {
        let mut dest = json!({
            "a": "b",
            "c": {
                "d": "e",
            },
            "f": {
                "g": "h",
                "i": [1_i32, 2_i32, 3_i32],
            },
        });
        let src = json!({
            "a": "x",
            "c": 123123_i32,
            "f": {
                "a": 42_i32,
                "g": "y",
                "i": [1_i32, 2_i32, 4_i32],
                "p": {
                    "x": "y",
                },
            },
            "z": "hello",
        });
        merge_templates(&mut dest, &src);
        assert_eq!(
            dest,
            json!({
                "a": "b",
                "c": {
                    "d": "e",
                },
                "f": {
                    "a": 42_i32,
                    "g": "h",
                    "i": [1_i32, 2_i32, 3_i32],
                    "p": {
                        "x": "y",
                    },
                },
                "z": "hello",
            }),
        );
    }

    #[test]
    fn merge_templates_deep() {
        let src = json!({"a": {"b": {"c": {"d": "e"}}}});

        let mut dest = json!({});
        merge_templates(&mut dest, &src);
        assert_eq!(dest, src);

        let mut dest = json!({"a": {"b": {"c": {"k": "l"}}}});
        merge_templates(&mut dest, &src);
        assert_eq!(dest, json!({"a": {"b": {"c": {"d": "e", "k": "l"}}}}));
    }
}
