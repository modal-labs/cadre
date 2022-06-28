//! Client for cadre.

use anyhow::{ensure, Result};
use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Request};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::env;

/// An asynchronous client for the file server.
#[derive(Clone)]
pub struct CadreClient {
    client: Client<HttpConnector>,
    origin: String,
}

pub const DEFAULT_ORIGIN: &str = "http://configs.modal.internal";

impl CadreClient {
    /// Create a new file client object pointing at a given HTTP origin.
    pub fn new(origin: Option<&str>) -> Self {
        let mut connector = HttpConnector::new();
        connector.set_nodelay(true);

        // use env var for overriding default origin
        let mut origin_value = String::new();
        if origin.is_none() {
            let cadre_url = option_env!("CADRE_URL");
            origin_value = cadre_url.unwrap_or(DEFAULT_ORIGIN).to_string();
        } else {
            origin_value = origin.unwrap().into();
        }

        Self {
            client: Client::builder().build(connector),
            origin: origin_value,
        }
    }

    async fn get(&self, uri: &str) -> Result<Value> {
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())?;

        let resp = self.client.request(req).await?;

        // check response status
        let status = resp.status();
        ensure!(!status.is_server_error(), "server error: {status}");
        ensure!(status.is_success(), "get request failed: {status}");

        // asynchronously aggregate the chunks of the body and create serde Value
        let body = hyper::body::aggregate(resp).await?;
        let json: Value = serde_json::from_reader(body.reader())?;

        Ok(json)
    }

    /// Rendered config object from cadre.
    pub async fn get_config(&self, environment: &str) -> Result<String> {
        let value = self
            .get(&format!("{}/c/{}", self.origin, environment))
            .await?;

        // evaluate if there's an env var override
        let env_vars = env_var_overrides()?;
        let flat_map = flatten(value.as_object().unwrap())?;
        let mut key_value = String::new();
        for (k, _v) in flat_map.iter() {
            for e in k {
                key_value += &format!("_{}", e);
            }
            let _env_override = match env_vars.get(&key_value) {
                Some(v) => {
                    println!("found env override: {}", v);
                }
                _ => {
                    println!("no override found")
                }
            };
        }

        Ok(value.to_string())
    }

    /// Fetch original templated JSON object.
    pub async fn get_template(&self, environment: &str) -> Result<String> {
        let value = self
            .get(&format!("{}/t/{}", self.origin, environment))
            .await?;

        Ok(value.to_string())
    }
}

// Collects environment variables that start with MODAL_ into a HashMap.
// We'll later use this to override existing config values.
fn env_var_overrides() -> Result<HashMap<String, String>> {
    let mut envs: HashMap<String, String> = HashMap::new();
    for (k, v) in env::vars() {
        let mut upper_k = k.to_ascii_uppercase();
        if upper_k.starts_with("MODAL") {
            upper_k = upper_k.replace("MODAL_", "");
            envs.insert(upper_k, v);
        }
    }

    Ok(envs)
}

// Flattens a serde Map. Useful for comparing and overriding key-value pairs.
fn flatten(map: &Map<String, Value>) -> Result<Vec<(Vec<String>, String)>> {
    let mut flat_map = Vec::new();
    for (key, value) in map.iter() {
        let mut key_v = vec![key.clone()];
        if value.is_object() {
            let mut inner_object = flatten(value.as_object().unwrap())?;
            for (k, v) in inner_object.iter_mut() {
                key_v.append(k);
                flat_map.push((key_v.clone(), v.to_string()));
            }
        } else {
            flat_map.push((key_v.clone(), value.to_string()));
        }
    }

    Ok(flat_map)
}
