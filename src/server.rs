//! Web server handler implementation using axum.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use anyhow::Result;
use axum::extract::{Extension, Path};
use axum::routing::get;
use axum::{http::StatusCode, response::Html, Json, Router};
use serde_json::Value;
use tracing::{error, warn};

use self::state::State;

pub mod resolver;
pub mod state;
pub mod storage;
pub mod template;

/// Web server for handling requests.
pub async fn server(bucket: &str, default_template: Option<&str>) -> Result<Router> {
    let state = State::new(bucket, default_template).await?;

    Ok(Router::new()
        .route("/", get(|| async { Html(include_str!("index.html")) }))
        .route("/t/:env", get(get_template_handler).put(put_handler))
        .route("/c", get(get_all_configs_handler))
        .route("/c/:env", get(get_config_handler))
        .route("/ping", get(|| async { "cadre ok" }))
        .layer(Extension(state)))
}

async fn get_template_handler(
    Extension(state): Extension<State>,
    Path(env): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match state.read_template(&env).await {
        Ok(value) => Ok(Json(value)),
        Err(err) => {
            warn!(%env, ?err, "problem getting template");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn get_config_handler(
    Extension(state): Extension<State>,
    Path(env): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match state.load_config(&env).await {
        Ok(value) => Ok(Json(value)),
        Err(err) => {
            warn!(%env, ?err, "problem reading config");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn get_all_configs_handler(
    Extension(state): Extension<State>,
) -> Result<Json<Value>, StatusCode> {
    match state.list_available_configs().await {
        Ok(value) => Ok(Json(value)),
        Err(err) => {
            warn!(?err, "problem reading all configs");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn put_handler(
    Extension(state): Extension<State>,
    Path(env): Path<String>,
    body: Json<Value>,
) -> Result<(), StatusCode> {
    match state.write_template(&env, &body).await {
        Ok(_) => Ok(()),
        Err(err) => {
            error!(?err, "could not put config");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
