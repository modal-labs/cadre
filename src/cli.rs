//! Command-line interface (used by the binary).

use std::net::{Ipv6Addr, SocketAddr};
use std::path::PathBuf;

use anyhow::{bail, Result};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client;
use aws_types::sdk_config::SdkConfig;
use clap::Parser;
use tracing::info;

use crate::server::resolver::{AwsSecrets, ResolverChain};
use crate::server::{server, state::State, storage::Storage};

/// Creates an AWS SDK default config object.
pub async fn default_aws_config() -> SdkConfig {
    let region_provider = RegionProviderChain::default_provider();
    aws_config::from_env().region(region_provider).load().await
}

/// A simple, self-hosted, high-performance remote configuration service.
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub struct Args {
    /// Port to serve cadre on.
    #[clap(short, long, default_value_t = 7608, env = "CADRE_PORT")]
    port: u16,

    /// Secret to verify clients against.
    #[clap(long, env = "CADRE_SECRET")]
    secret: String,

    /// S3 bucket to use for persisting template JSON files.
    #[clap(long, env = "CADRE_BUCKET")]
    bucket: Option<String>,

    /// Local directory to use for persisting template JSON files.
    #[clap(long, parse(from_os_str), env = "CADRE_LOCAL_DIR")]
    local_dir: Option<PathBuf>,

    /// Sets a default templated JSON to be used for other environments
    /// to build upon. Ignored if left empty.
    #[clap(long, env = "CADRE_DEFAULT_TEMPLATE")]
    default_template: Option<String>,
}

impl Args {
    /// Run the action corresponding to this CLI command.
    pub async fn run(self) -> Result<()> {
        let sdk_config = default_aws_config().await;

        let mut chain = ResolverChain::new();
        chain.add(AwsSecrets::new(&sdk_config));

        let storage = match (&self.bucket, &self.local_dir) {
            (Some(bucket), None) => Storage::S3(Client::new(&sdk_config), bucket.into()),
            (None, Some(local_dir)) => Storage::LocalFS(local_dir.into()),
            _ => bail!("must specify exactly one of --bucket or --local-dir"),
        };

        let logged_secret = self.secret.clone();
        let state = State::new(chain, storage, self.default_template.as_deref());
        let app = server(state, self.secret);

        let addr: SocketAddr = (Ipv6Addr::UNSPECIFIED, self.port).into();
        info!(?addr, "running cadre");
        info!("visit frontend at {}?secret={}", addr, logged_secret);

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}
