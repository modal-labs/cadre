//! Implementation of the Rust client for cadre.

use anyhow::{ensure, Result};
use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Client, Request};
use serde::de::DeserializeOwned;
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

    async fn send(&self, req: Request<Body>) -> Result<impl Buf> {
        let resp = self.client.request(req).await?;

        // check response status
        let status = resp.status();
        ensure!(!status.is_server_error(), "cadre server error: {status}");
        ensure!(status.is_success(), "cadre get request failed: {status}");

        // asynchronously aggregate the chunks of the body and create serde json
        Ok(hyper::body::aggregate(resp).await?)
    }

    async fn get<T: DeserializeOwned>(&self, uri: &str) -> Result<T> {
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())?;
        let resp = self.send(req).await?;
        Ok(serde_json::from_reader(resp.reader())?)
    }

    /// Fetch the raw JSON source for a template.
    pub async fn read_template(&self, env: &str) -> Result<Value> {
        self.get(&format!("{}/t/{}", self.origin, env)).await
    }

    /// Write the value for a template.
    pub async fn write_template(&self, env: &str, template: &Value) -> Result<()> {
        let req = Request::builder()
            .method("PUT")
            .uri(format!("{}/t/{}", self.origin, env))
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(template)?.into())?;
        self.send(req).await?;
        Ok(())
    }

    /// Read a populated configuration with templated and default values.
    pub async fn load_config(&self, env: &str) -> Result<Value> {
        self.get(&format!("{}/c/{}", self.origin, env)).await
    }

    /// List all available configuration environment names.
    pub async fn list_configs(&self) -> Result<Vec<String>> {
        self.get(&format!("{}/c", self.origin)).await
    }
}
