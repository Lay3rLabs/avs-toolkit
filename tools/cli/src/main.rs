mod args;
mod commands;
mod config;
mod context;

use anyhow::Result;
use args::{CliArgs, Command, DeployCommand, TaskQueueCommand};
use clap::Parser;
use commands::{deploy::deploy_contracts, task_queue::TaskQueue};
use context::AppContext;
use layer_climb_cli::command::{ContractLog, WalletLog};

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
        Command::Deploy(deploy_args) => match deploy_args.command {
            DeployCommand::Contracts { artifacts_path } => {
                let addrs = deploy_contracts(ctx, artifacts_path).await?;
                tracing::info!("---- All contracts instantiated successfully ----");
                tracing::info!("Mock Operators: {}", addrs.operators);
                tracing::info!("Verifier Simple: {}", addrs.verifier_simple);
                tracing::info!("Task Queue: {}", addrs.task_queue);
            }
        },
        Command::TaskQueue(task_queue_args) => {
            let task_queue = TaskQueue::new(ctx.clone(), &task_queue_args).await?;

            match task_queue_args.command {
                TaskQueueCommand::AddTask {
                    body,
                    description,
                    timeout,
                } => {
                    let _ = task_queue.add_task(body, description, timeout).await?;
                }
                TaskQueueCommand::ViewQueue => {
                    let res = task_queue.querier.task_queue_view().await?;
                    tracing::info!("Task Queue Configuration");
                    res.report(|line| {
                        println!("{}", line);
                    })?;
                }
            }
        }
        Command::Wallet(wallet_args) => {
            let mut rng_lock = ctx.rng.lock().await;
            wallet_args
                .command
                .run(ctx.any_client().await?, &mut *rng_lock, |line| match line {
                    WalletLog::Create { addr, mnemonic } => {
                        tracing::info!("--- Address ---");
                        tracing::info!("{}", addr);
                        tracing::info!("--- Mnemonic ---");
                        tracing::info!("{}", mnemonic);
                    }
                    WalletLog::Show { addr, balances } => {
                        tracing::info!("Wallet address: {}", addr);
                        for balance in balances {
                            tracing::info!("{}: {}", balance.denom, balance.amount);
                        }
                    }
                    WalletLog::Transfer {
                        to,
                        amount,
                        denom,
                        tx_resp,
                    } => {
                        tracing::info!("Transfer successful, tx hash: {}", tx_resp.txhash);
                        tracing::info!("Sent {} {} to {}", amount, denom, to);
                    }
                })
                .await?;
        }

        Command::Contract(contract_args) => {
            contract_args
                .command
                .run(ctx.signing_client().await?, |line| match line {
                    ContractLog::Upload { code_id, tx_resp } => {
                        tracing::info!("Uploaded contract with code id: {}", code_id);
                        tracing::debug!("Tx hash: {}", tx_resp.txhash);
                    }
                    ContractLog::Instantiate { addr, tx_resp } => {
                        tracing::info!("Instantiated contract at address: {}", addr);
                        tracing::debug!("Tx hash: {}", tx_resp.txhash);
                    }
                    ContractLog::Execute { tx_resp } => {
                        tracing::info!("Executed contract, tx hash: {}", tx_resp.txhash);
                    }
                    ContractLog::Query { response } => {
                        tracing::info!("Contract query response: {}", response);
                    }
                })
                .await?;
        }
    }

    Ok(())
}
