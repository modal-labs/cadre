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

/// Specification of each subcommand used by the worker.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the cadre service
    Server,
}

impl Cli {
    /// Run the action corresponding to this CLI command.
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::Server => run_server().await?,
        }
        Ok(())
    }
}

async fn run_server() -> Result<()> {
    let server_addr = String::from("0.0.0.0:3000");
    println!(" => running cadre at: {}", server_addr);
    let app = server().await?;
    axum::Server::bind(&server_addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
