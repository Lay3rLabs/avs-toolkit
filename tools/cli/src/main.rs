mod args;
mod commands;
mod config;
mod context;

use anyhow::{Context, Result};
use args::{
    CliArgs, Command, DeployCommand, FaucetCommand, TargetEnvironment, TaskQueueCommand,
    WasmaticCommand,
};
use avs_toolkit_shared::{faucet::tap_faucet, task_queue::TaskQueue, wasmatic};
use clap::Parser;
use commands::{
    deploy::{deploy_contracts, DeployContractArgs},
    wasmatic::wasm_arg_to_file,
};
use context::AppContext;
use layer_climb::prelude::*;
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
            DeployCommand::Contracts {
                artifacts_path,
                timeout: task_timeout_seconds,
                percentage: required_voting_percentage,
                operators,
                requestor,
            } => {
                let args = DeployContractArgs::parse(
                    &ctx,
                    artifacts_path,
                    task_timeout_seconds,
                    required_voting_percentage,
                    operators,
                    requestor,
                )
                .await?;

                let addrs = deploy_contracts(ctx, args).await?;
                tracing::info!("---- All contracts instantiated successfully ----");
                tracing::info!("Mock Operators: {}", addrs.operators);
                tracing::info!("Verifier Simple: {}", addrs.verifier_simple);
                tracing::info!("Task Queue: {}", addrs.task_queue);
                // TODO: make a flag to select one of these
                tracing::info!("export LOCAL_TASK_QUEUE_ADDRESS={}", addrs.task_queue);
                tracing::info!("export TEST_TASK_QUEUE_ADDRESS={}", addrs.task_queue);
            }
        },
        Command::TaskQueue(task_queue_args) => {
            let addr_string = match task_queue_args.address.clone() {
                Some(x) => x,
                None => match ctx.args.target {
                    TargetEnvironment::Local => std::env::var("LOCAL_TASK_QUEUE_ADDRESS")
                        .context("LOCAL_TASK_QUEUE_ADDRESS not found")?,
                    TargetEnvironment::Testnet => std::env::var("TEST_TASK_QUEUE_ADDRESS")
                        .context("TEST_TASK_QUEUE_ADDRESS not found")?,
                },
            };

            let contract_addr = ctx.chain_config()?.parse_address(&addr_string)?;

            let task_queue = TaskQueue::new(ctx.signing_client().await?, contract_addr).await?;

            match task_queue_args.command {
                TaskQueueCommand::AddTask {
                    body,
                    description,
                    timeout,
                } => {
                    let _ = task_queue.add_task(body, description, timeout).await?;
                }
                TaskQueueCommand::ViewQueue { start_after, limit } => {
                    let res = task_queue
                        .querier
                        .task_queue_view(start_after, limit)
                        .await?;
                    tracing::info!("Task Queue Configuration");
                    tracing::info!("Address: {}", task_queue.contract_addr);
                    res.report(|line| {
                        println!("{}", line);
                    })?;
                }
            }
        }
        Command::Faucet(faucet_args) => match faucet_args.command {
            FaucetCommand::Tap { to, amount, denom } => {
                let to = match to {
                    Some(to) => ctx.chain_config()?.parse_address(&to)?,
                    None => ctx.any_client().await?.as_signing().addr.clone(),
                };

                let amount = amount.unwrap_or(FaucetCommand::DEFAULT_TAP_AMOUNT);
                tap_faucet(ctx.faucet_client().await?, to, amount, denom).await?;
            }
        },
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
                    WalletLog::Balance { addr, balance } => {
                        tracing::info!("Wallet address: {}", addr);
                        tracing::info!("{}: {}", balance.denom, balance.amount);
                    }
                    WalletLog::AllBalances { addr, balances } => {
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
        Command::Wasmatic(wasmatic_args) => match wasmatic_args.command {
            WasmaticCommand::Deploy {
                name,
                digest,
                wasm_source,
                cron_trigger,
                task_trigger,
                hd_index,
                poll_interval,
                permissions,
                envs,
                testable,
            } => {
                let trigger = match (cron_trigger, task_trigger) {
                    (Some(cron), None) => wasmatic::Trigger::Cron { schedule: cron },
                    (None, Some(task)) => wasmatic::Trigger::Queue {
                        task_queue_addr: task,
                        hd_index,
                        poll_interval,
                    },
                    _ => {
                        panic!("Error: You need to provide either cron_trigger or task_trigger")
                    }
                };
                wasmatic::deploy(
                    reqwest::Client::new(),
                    ctx.chain_info()?.wasmatic.endpoints.clone(),
                    name,
                    digest,
                    wasm_arg_to_file(wasm_source).await?,
                    trigger,
                    permissions,
                    envs,
                    testable,
                    |endpoint| {
                        println!("Deployment successful to: {endpoint}");
                    },
                )
                .await?;
            }
            WasmaticCommand::Remove { name } => {
                wasmatic::remove(
                    reqwest::Client::new(),
                    ctx.chain_info()?.wasmatic.endpoints.clone(),
                    name,
                    |endpoint| {
                        println!("Removal successful from: {endpoint}");
                    },
                )
                .await?;
            }
            WasmaticCommand::Run {
                wasm_source,
                cron_trigger,
                envs,
                dir,
                input,
            } => {
                let app_cache_path = if let Some(dir) = dir {
                    dir
                } else {
                    tempfile::tempdir()
                        .context("failed to create temp directory for app cache")?
                        .path()
                        .into()
                };
                let wasm_file = wasm_arg_to_file(wasm_source).await?;
                println!(
                    "{}",
                    commands::wasmatic::run(wasm_file, cron_trigger, envs, app_cache_path, input)
                        .await?
                );
            }
            WasmaticCommand::Test { name, input } => {
                wasmatic::test(
                    reqwest::Client::new(),
                    ctx.chain_info()?.wasmatic.endpoints.clone(),
                    name,
                    input,
                    |wasmatic::TestResult {
                         endpoint,
                         response_text,
                     }| {
                        println!("Test executed successfully!");
                        println!("Output for operator `{endpoint}`: {}", response_text);
                    },
                )
                .await?;
            }
            WasmaticCommand::Info {} => {
                wasmatic::info(
                    reqwest::Client::new(),
                    ctx.chain_info()?.wasmatic.endpoints.clone(),
                    |wasmatic::InfoResponse { endpoint, response }| {
                        println!(
                            "Output for operator `{endpoint}`: {}",
                            serde_json::to_string_pretty(&response).unwrap()
                        );
                    },
                )
                .await?;
            }
            WasmaticCommand::App { endpoint } => {
                let endpoint = endpoint.unwrap_or_else(|| {
                    ctx.chain_info()
                        .unwrap()
                        .wasmatic
                        .endpoints
                        .first()
                        .unwrap()
                        .clone()
                });

                wasmatic::app(reqwest::Client::new(), endpoint).await?;
            }
        },
    }

    Ok(())
}
