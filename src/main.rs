use std::process;

use cadre::Args;
use clap::Parser;
use tracing::error;

/// Main entry point for the `cadre` binary.
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    match Args::parse().run().await {
        Ok(()) => process::exit(0),
        Err(err) => {
            error!("{err:?}");
            process::exit(1)
        }
    }
}
