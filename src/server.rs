//! Web server handler implementation using axum.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use anyhow::Result;
use axum::extract::{Extension, Path};
use axum::routing::{get, put};
use axum::{http::StatusCode, response::Html, Json, Router};
use serde_json::Value;
use tracing::{error, warn};

use crate::storage::Storage;

/// Web server for handling requests.
pub async fn server(bucket: &str, default_template: Option<&str>) -> Result<Router> {
    let storage = Storage::new(bucket, default_template).await?;

    Ok(Router::new()
        .route("/", get(|| async { Html(include_str!("index.html")) }))
        .route("/w/:env", put(put_handler))
        .route("/t/:env", get(get_template_handler))
        .route("/c", get(get_all_configs_handler))
        .route("/c/:env", get(get_config_handler))
        .route("/ping", get(|| async { "cadre ok" }))
        .layer(Extension(storage)))
}

async fn get_template_handler(
    Extension(storage): Extension<Storage>,
    Path(env): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match storage.read_template(&env).await {
        Ok(value) => Ok(Json(value)),
        Err(err) => {
            warn!(%env, ?err, "problem getting template");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn get_config_handler(
    Extension(storage): Extension<Storage>,
    Path(env): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match storage.read_config(&env).await {
        Ok(value) => Ok(Json(value)),
        Err(err) => {
            warn!(%env, ?err, "problem reading config");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn get_all_configs_handler(
    Extension(storage): Extension<Storage>,
) -> Result<Json<Value>, StatusCode> {
    match storage.list_available_configs().await {
        Ok(value) => Ok(Json(value)),
        Err(err) => {
            warn!(?err, "problem reading all configs");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn put_handler(
    Extension(storage): Extension<Storage>,
    Path(env): Path<String>,
    body: Json<Value>,
) -> Result<(), StatusCode> {
    match storage.write(&env, &body).await {
        Ok(_) => Ok(()),
        Err(err) => {
            error!(?err, "could not put config");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
