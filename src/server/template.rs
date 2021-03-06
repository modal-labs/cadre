//! Engine for populating cadre configuration templates.

use anyhow::Context;
use anyhow::Result;
use serde_json::Value;

use super::resolver::ResolverChain;

/// Marker character used for template strings.
pub const TEMPLATE_MARK: &str = "*";

/// Populate a JSON value with the results of templated strings.
pub async fn populate_template(value: &mut Value, chain: &ResolverChain) -> Result<()> {
    let mut stack = vec![value];
    while let Some(value) = stack.pop() {
        if let Some(map) = value.as_object_mut() {
            for (key, value) in std::mem::take(map) {
                if let Some(key_raw) = key.strip_prefix(TEMPLATE_MARK) {
                    let value = value
                        .as_str()
                        .with_context(|| format!("templated key {key:?} is of non-string type"))?;
                    map.insert(key_raw.into(), chain.resolve(value).await?);
                } else {
                    map.insert(key, value);
                }
            }
            stack.extend(map.values_mut());
        }
    }
    Ok(())
}

/// Merge two raw template objects, remaining aware of unpopulated values.
///
/// This does not overwrite data. If a property is present in both the
/// destination template and the source template, then the destination
/// template's value will be preserved.
pub fn merge_templates(dest: &mut Value, src: &Value) {
    if let (Value::Object(dest), Value::Object(src)) = (dest, src) {
        for (key, value) in src {
            if !dest.contains_key(key) {
                // Total replacement: dest does not contain the key.
                dest.insert(key.clone(), value.clone());
            } else if dest.contains_key(key) {
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

    use super::{merge_templates, populate_template};
    use crate::server::resolver::{EchoJson, ResolverChain};

    #[test]
    fn merge_templates_basic() {
        let mut dest = json!({ "a": "b" });
        let src = json!({ "c": "d" });
        merge_templates(&mut dest, &src);
        assert_eq!(dest, json!({ "a": "b", "c": "d" }));
    }

    #[tokio::test]
    async fn merge_template_with_populate() {
        let mut chain = ResolverChain::new();
        chain.add(EchoJson);

        // Templates are merged correctly when base templated keys exist.
        let mut dest = json!({ "*a": "echo:\"hello\"" });
        let src = json!({ "c": "d" });

        populate_template(&mut dest, &chain).await.unwrap();
        merge_templates(&mut dest, &src);
        assert_eq!(dest, json!({ "a": "hello", "c": "d" }));

        // Nested templates are merged superimposing keys from destination,
        // event if the latter is templated.
        let mut dest = json!({
            "a": {
                "*b": "echo:\"hello\""
            }
        });
        let src = json!({ "a": {
            "b": "foo",
            "c": "bar",
        } });

        populate_template(&mut dest, &chain).await.unwrap();
        merge_templates(&mut dest, &src);
        assert_eq!(
            dest,
            json!({ "a": {
            "b": "hello",
            "c": "bar",
        } })
        );
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

    #[tokio::test]
    async fn populate_with_resolver() {
        let mut chain = ResolverChain::new();
        chain.add(EchoJson);

        let mut value = json!({"a": {"value": "3", "*rep": "echo:\"hello\""}});
        populate_template(&mut value, &chain).await.unwrap();
        assert_eq!(value, json!({"a": {"value": "3", "rep": "hello"}}));

        // Recursive resolvers
        let mut value = json!({"*a": "echo:{\"*b\": \"echo:4\"}"});
        populate_template(&mut value, &chain).await.unwrap();
        assert_eq!(value, json!({"a": {"b": 4}}));
    }

    #[tokio::test]
    async fn fail_populate() {
        let chain = ResolverChain::new();

        let mut value = json!({"*missing": "foo:bar"});
        assert!(populate_template(&mut value, &chain).await.is_err());

        let mut value = json!({"*invalid": "$@not@avalidliteral"});
        assert!(populate_template(&mut value, &chain).await.is_err());
    }
}
