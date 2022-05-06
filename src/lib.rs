//! Cadre is a simple, self-hosted, high-performance, and strongly consistent
//! remote configuration service.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use anyhow::Result;
use axum::extract::{Extension, Path};
use axum::{http::StatusCode, response::Html, routing::get, Json, Router};
use serde_json::Value;

use crate::storage::Storage;

mod storage;

/// Web server for handling requests.
pub async fn server() -> Result<Router> {
    let storage = Storage::new().await?;

    Ok(Router::new()
        .route("/", get(|| async { Html(include_str!("index.html")) }))
        .route("/p/*path", get(get_handler).put(put_handler))
        .layer(Extension(storage)))
}

async fn get_handler(
    Extension(storage): Extension<Storage>,
    path: Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let path = path.trim_end_matches('/');
    match storage.read().await.pointer(path) {
        Some(value) => Ok(Json(value.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn put_handler(
    Extension(storage): Extension<Storage>,
    path: Path<String>,
    body: Json<Value>,
) -> StatusCode {
    let path = path.trim_end_matches('/');
    if !path.is_empty() {
        // Can only write to the root path `/p`, replacing all configuration.
        return StatusCode::NOT_FOUND;
    }
    match storage.write(&body).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
