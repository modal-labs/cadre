//! Web server handler implementation using axum.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use anyhow::Result;
use axum::extract::{Extension, Path};
use axum::http::{Request, StatusCode};
use axum::middleware;
use axum::response::Response;
use axum::routing::get;
use axum::{response::Html, Json, Router};
use serde_json::Value;
use tracing::{error, warn};

use self::state::State;

pub mod cache;
pub mod resolver;
pub mod state;
pub mod storage;
pub mod template;

/// Authorization token validation.
async fn auth<B>(
    req: Request<B>,
    next: middleware::Next<B>,
    secret: String,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("X-Cadre-Secret")
        .and_then(|header| header.to_str().ok());

    // Checks auth header against secret.
    match auth_header {
        Some(auth_header) if auth_header == secret => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Web server for handling requests.
pub fn server(state: State, secret: String) -> Router {
    Router::new()
        .route("/", get(|| async { Html(include_str!("index.html")) }))
        .route("/t/:env", get(get_template_handler).put(put_handler))
        .route("/c", get(list_configs_handler))
        .route("/c/:env", get(get_config_handler))
        .layer(Extension(state))
        .route_layer(middleware::from_fn(move |req, next| {
            auth(req, next, secret.clone())
        }))
        .route("/ping", get(|| async { "cadre ok" }))
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

async fn list_configs_handler(
    Extension(state): Extension<State>,
) -> Result<Json<Vec<String>>, StatusCode> {
    match state.list_configs().await {
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
