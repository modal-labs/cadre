//! Cadre is a simple, self-hosted, high-performance, and strongly consistent
//! remote configuration service.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::HashMap;

use anyhow::Result;
use axum::extract::{Extension, Path};
use axum::routing::{get, put};
use axum::{http::StatusCode, response::Html, Json, Router};
use serde_json::Value;

use crate::storage::Storage;

/// Web server for handling requests.
pub async fn server(bucket: String, default_template: Option<String>) -> Result<Router> {
    let storage = Storage::new(bucket.parse()?, default_template).await?;

    Ok(Router::new()
        .route("/", get(|| async { Html(include_str!("index.html")) }))
        .route("/w/:environment", put(put_handler))
        .route("/t/:environment", get(get_template_handler))
        .route("/c", get(get_all_configs_handler))
        .route("/c/:environment", get(get_config_handler))
        .route("/ping", get(get_ping_handler))
        .layer(Extension(storage)))
}

async fn get_template_handler(
    Extension(storage): Extension<Storage>,
    Path(params): Path<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    if let Some(environment) = params.get("environment") {
        match storage.read_template(environment).await {
            Ok(value) => Ok(Json(value)),
            Err(error) => {
                println!("error: {}", error);
                Err(StatusCode::NOT_FOUND)
            }
        }
    } else {
        Err(StatusCode::NOT_ACCEPTABLE)
    }
}

async fn get_config_handler(
    Extension(storage): Extension<Storage>,
    Path(params): Path<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    if let Some(environment) = params.get("environment") {
        match storage.read_parsed_template(environment).await {
            Ok(value) => Ok(Json(value)),
            Err(error) => {
                println!("error: {}", error);
                Err(StatusCode::NOT_FOUND)
            }
        }
    } else {
        Err(StatusCode::NOT_ACCEPTABLE)
    }
}

async fn get_all_configs_handler(
    Extension(storage): Extension<Storage>,
) -> Result<Json<Value>, StatusCode> {
    match storage.list_available_configs().await {
        Ok(value) => Ok(Json(value)),
        Err(error) => {
            println!("error: {}", error);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn put_handler(
    Extension(storage): Extension<Storage>,
    Path(params): Path<HashMap<String, String>>,
    body: Json<Value>,
) -> Result<StatusCode, StatusCode> {
    if let Some(environment) = params.get("environment") {
        match storage.write(environment, &body).await {
            Ok(()) => Ok(StatusCode::OK),
            Err(error) => {
                println!("{}", error);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::NOT_ACCEPTABLE)
    }
}

async fn get_ping_handler() -> Result<Json<Value>, StatusCode> {
    let message = String::from("OK");
    Ok(Json(Value::from(message)))
}
