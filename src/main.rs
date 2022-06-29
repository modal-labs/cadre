use std::process::ExitCode;

use cadre::Args;
use clap::Parser;
use tracing::error;

/// Main entry point for the `cadre` binary.
#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt::init();

    match Args::parse().run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{err:?}");
            ExitCode::FAILURE
        }
    }
}
