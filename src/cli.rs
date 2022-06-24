//! Implementation of the cadre command-line interface.

use crate::server::server;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    /// Commands supported by the CLI.
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the cadre service
    Server {
        /// Port to serve cadre on.
        #[clap(short, long, value_parser, default_value = "3000")]
        port: String,

        /// Bucket to use for storing cadre templated JSON files.
        #[clap(short, long, value_parser)]
        bucket: String,

        /// Sets a default templated JSON to be used for other environments
        /// to build upon. Ignored if left empty.
        #[clap(short, long, value_parser)]
        default_template: Option<String>,
    },
}

impl Cli {
    /// Run the action corresponding to this CLI command.
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::Server {
                port,
                bucket,
                default_template,
            } => run_server(port, bucket, default_template).await?,
        }
        Ok(())
    }
}

async fn run_server(port: String, bucket: String, default_template: Option<String>) -> Result<()> {
    let server_addr = format!("0.0.0.0:{}", port);
    println!(" => running cadre at: {}", server_addr);
    let app = server(bucket, default_template).await?;
    axum::Server::bind(&server_addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
