mod args;

use anyhow::Result;
use args::Command;
use clap::Parser;
use lavs_orch::daemon::slay3r_connect;
use tracing;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // load .env file first so it's available for clap
    let dotenv_resp = dotenvy::dotenv();

    // parse cli args first so we can get logs even while getting full args
    let args = args::CliArgs::parse();

    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_max_level(tracing::Level::from(args.log_level))
        .init();

    if dotenv_resp.is_err() {
        tracing::warn!("No .env file found");
    }

    let args = args::Args::new(args).await?;

    match args.command {
        Command::DeployAvsContracts {  } => {

            tracing::info!("connectiing...");
            let daemon = slay3r_connect(args.chain_kind.into());
            tracing::info!("{:?}", daemon.chain_info());
        }
    }

    Ok(())
}