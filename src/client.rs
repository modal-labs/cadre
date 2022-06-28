//! Client for cadre.

use anyhow::{ensure, Result};
use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Request};
use serde_json::Value;

/// An asynchronous client for the file server.
#[derive(Clone)]
pub struct CadreClient {
    client: Client<HttpConnector>,
    origin: String,
}

pub const DEFAULT_ORIGIN: &str = "http://configs.modal.internal";

impl CadreClient {
    /// Create a new file client object pointing at a given HTTP origin.
    #[allow(clippy::unnecessary_unwrap)]
    pub fn new(origin: Option<&str>) -> Self {
        let mut connector = HttpConnector::new();
        connector.set_nodelay(true);

        // use env var for overriding default origin
        let origin_value: String = if origin.is_none() {
            let cadre_url = option_env!("CADRE_URL");
            cadre_url.unwrap_or(DEFAULT_ORIGIN).to_string()
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

    /// Rendered config object from cadre.
    pub async fn get_config(&self, environment: &str) -> Result<String> {
        self.get(&format!("{}/c/{}", self.origin, environment))
            .await
    }

    /// Fetch original templated JSON object.
    pub async fn get_template(&self, environment: &str) -> Result<String> {
        self.get(&format!("{}/t/{}", self.origin, environment))
            .await
    }
}
