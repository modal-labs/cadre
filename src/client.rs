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
    pub fn new(origin: &str) -> Self {
        let mut connector = HttpConnector::new();
        connector.set_nodelay(true);

        Self {
            client: Client::builder().build(connector),
            origin: origin.into(),
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
