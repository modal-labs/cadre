//! Cadre is a simple, self-hosted, high-performance, and strongly consistent
//! remote configuration service.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use anyhow::Result;
use axum::extract::{Extension, Path};
use axum::routing::{get, put};
use axum::{http::StatusCode, response::Html, Json, Router};
use serde_json::Value;

use crate::storage::Storage;

mod storage;

/// Web server for handling requests.
pub async fn server() -> Result<Router> {
    let storage = storage::Storage::new("modal-cadre-config-store-dev".parse()?).await?;

    Ok(Router::new()
        .route("/", get(|| async { Html(include_str!("index.html")) }))
        .route("/p/*environment", get(get_handler))
        .route("/w", put(put_handler))
        .layer(Extension(storage)))
}

// TODO: finish read handler
async fn get_handler(
    Extension(storage): Extension<Storage>,
    path: Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let path = path.as_str();
    match storage.read(path).await {
        Some(value) => Ok(Json(value.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn put_handler(Extension(storage): Extension<Storage>, body: Json<Value>) -> StatusCode {
    match storage.write(&body).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
