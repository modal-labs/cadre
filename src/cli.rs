//! Command-line interface (used by the binary).

use std::net::{Ipv6Addr, SocketAddr};

use anyhow::Result;
use clap::Parser;
use tracing::info;

use crate::server::server;

/// A simple, self-hosted, high-performance remote configuration service.
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub struct Args {
    /// Port to serve cadre on.
    #[clap(short, long, default_value_t = 3000)]
    port: u16,

    /// S3 bucket to use for storing cadre templated JSON files.
    #[clap(short, long)]
    bucket: String,

    /// Sets a default templated JSON to be used for other environments
    /// to build upon. Ignored if left empty.
    #[clap(short, long)]
    default_template: Option<String>,
}

impl Args {
    /// Run the action corresponding to this CLI command.
    pub async fn run(self) -> Result<()> {
        run_server(self.port, &self.bucket, self.default_template.as_deref()).await
    }
}

async fn run_server(port: u16, bucket: &str, default_template: Option<&str>) -> Result<()> {
    let app = server(bucket, default_template).await?;

    let addr: SocketAddr = (Ipv6Addr::UNSPECIFIED, port).into();
    info!(?addr, "running cadre");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
