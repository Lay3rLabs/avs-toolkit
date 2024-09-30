mod args;
mod commands;
mod config;
mod context;

use anyhow::Result;
use args::{CliArgs, Command};
use clap::Parser;
use commands::deploy::deploy_contracts;
use context::AppContext;

#[tokio::main]
async fn main() -> Result<()> {
    // Load the .env file before anything, in case it's used by args
    if dotenvy::dotenv().is_err() {
        println!("Failed to load .env file");
    }

    // load the args before setting up the logger, since it uses the log level
    let args = CliArgs::parse();

    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_max_level(tracing::Level::from(args.log_level))
        .init();

    // now we can get our context, which will contain the args too
    let ctx = AppContext::new(args).await?;

    match ctx.args.command.clone() {
        Command::DeployContracts { artifacts_path } => {
            deploy_contracts(ctx, artifacts_path).await?;
        }
        Command::TaskQueue(_task_queue_args) => {}
    }

    Ok(())
}
