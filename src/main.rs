use std::process;

use cadre::cli::Args;
use clap::Parser;
use tracing::error;

/// Main entry point for the `cadre` binary.
#[tokio::main]
async fn main() {
    match Args::parse().run().await {
        Ok(()) => process::exit(0),
        Err(err) => {
            error!("{err:?}");
            process::exit(1)
        }
    }
}
