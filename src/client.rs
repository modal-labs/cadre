//! Implementation of the Rust client for cadre.

use anyhow::{ensure, Result};
use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Request};
use serde_json::Value;

/// An asynchronous client for the configuration store.
#[derive(Clone)]
pub struct CadreClient {
    client: Client<HttpConnector>,
    origin: String,
}

impl CadreClient {
    /// Create a new client object pointing at a given HTTP origin.
    #[allow(clippy::unnecessary_unwrap)]
    pub fn new(origin: Option<&str>) -> Self {
        let mut connector = HttpConnector::new();
        connector.set_nodelay(true);

        // use env var for overriding default origin
        let origin_value: String = if origin.is_none() {
            let cadre_url = match option_env!("CADRE_URL") {
                Some(v) => v,
                None => panic!(
                    "No available cadre URL. Either pass 'origin' or use env var 'CADRE_URL'."
                ),
            };
            cadre_url.to_string()
        } else {
            origin.unwrap().into()
        };

        Self {
            client: Client::builder().build(connector),
            origin: origin_value,
        }
    }

    async fn get(&self, uri: &str) -> Result<String> {
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())?;

        let resp = self.client.request(req).await?;

        // check response status
        let status = resp.status();
        ensure!(!status.is_server_error(), "server error: {status}");
        ensure!(status.is_success(), "get request failed: {status}");

        // asynchronously aggregate the chunks of the body and create serde json
        let body = hyper::body::aggregate(resp).await?;
        let json: Value = serde_json::from_reader(body.reader())?;

        Ok(json.to_string())
    }

    /// Retrieve a populated config object.
    pub async fn get_config(&self, environment: &str) -> Result<String> {
        self.get(&format!("{}/c/{}", self.origin, environment))
            .await
    }

    /// Fetch the raw JSON value for a template.
    pub async fn get_template(&self, environment: &str) -> Result<String> {
        self.get(&format!("{}/t/{}", self.origin, environment))
            .await
    }
}
