#![allow(warnings)]
mod args;

use anyhow::{anyhow, bail, Context, Result};
use args::{Args, CliArgs, Command};
use clap::Parser;
use lavs_task_queue::msg::{Requestor, TimeoutInfo};
use layer_climb::prelude::*;
use std::{fs, os::unix::net};
use tracing;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Load the .env file before anything, in case it's used by CliArgs
    if dotenvy::dotenv().is_err() {
        println!("Failed to load .env file");
    }

    // load the CliArgs before setting up the logger, since it uses the log level
    let cli_args = CliArgs::parse();

    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_max_level(tracing::Level::from(cli_args.log_level))
        .init();

    // now we can get our real args :)
    let args = Args::new(cli_args).await?;

    let Args { command, client } = args;

    match &command {
        Command::DeployContracts { artifacts_path } => {
            tracing::info!("Uploading contracts from {:?}", artifacts_path);
            let operators_path = artifacts_path.join("lavs_mock_operators.wasm");
            let task_queue_path = artifacts_path.join("lavs_task_queue.wasm");
            let verifier_simple_path = artifacts_path.join("lavs_verifier_simple.wasm");

            if !operators_path.exists() {
                bail!(
                    "Mock Operators contract not found at {} (try running collect_wasm.sh)",
                    operators_path.display()
                );
            }
            if !task_queue_path.exists() {
                bail!(
                    "Task Queue contract not found at {} (try running collect_wasm.sh)",
                    task_queue_path.display()
                );
            }
            if !verifier_simple_path.exists() {
                bail!(
                    "Verifier Simple contract not found at {} (try running collect_wasm.sh)",
                    verifier_simple_path.display()
                );
            }

            // TODO - fix account sequence stuff in climb so we can do this all concurrently
            let operators_wasm = tokio::fs::read(operators_path).await?;
            let (operators_code_id, tx_resp) =
                client.contract_upload_file(operators_wasm, None).await?;
            tracing::info!("Mock Operators Tx Hash: {}", tx_resp.txhash);
            tracing::info!("Mock Operators Code ID: {}", operators_code_id);

            let task_queue_wasm = tokio::fs::read(task_queue_path).await?;
            let (task_queue_code_id, tx_resp) =
                client.contract_upload_file(task_queue_wasm, None).await?;
            tracing::info!("Task Queue Tx Hash: {}", tx_resp.txhash);
            tracing::info!("Task Queue Code ID: {}", task_queue_code_id);

            let verifier_simple_wasm = tokio::fs::read(verifier_simple_path).await?;
            let (verifier_code_id, tx_resp) = client
                .contract_upload_file(verifier_simple_wasm, None)
                .await?;
            tracing::info!("Verifier Simple Tx Hash: {}", tx_resp.txhash);
            tracing::info!("Verifier Simple Code ID: {}", verifier_code_id);

            tracing::info!("Contracts all uploaded successfully, instantiating...");

            let (operators_addr, tx_resp) = client
                .contract_instantiate(
                    client.addr.clone(),
                    operators_code_id,
                    "Mock Operators",
                    &lavs_mock_operators::msg::InstantiateMsg {
                        operators: vec![lavs_mock_operators::msg::InstantiateOperator::new(
                            client.addr.to_string(),
                            1,
                        )],
                    },
                    vec![],
                    None,
                )
                .await?;

            tracing::info!("Mock Operators Tx Hash: {}", tx_resp.txhash);
            tracing::info!("Mock Operators Address: {}", operators_addr);

            let (verifier_addr, tx_resp) = client
                .contract_instantiate(
                    client.addr.clone(),
                    verifier_code_id,
                    "Verifier Simple",
                    &lavs_verifier_simple::msg::InstantiateMsg {
                        operator_contract: operators_addr.to_string(),
                        required_percentage: 1,
                    },
                    vec![],
                    None,
                )
                .await?;

            tracing::info!("Verifier Simple Tx Hash: {}", tx_resp.txhash);
            tracing::info!("Verifier Simple Address: {}", verifier_addr);

            let (task_queue_addr, tx_resp) = client
                .contract_instantiate(
                    client.addr.clone(),
                    task_queue_code_id,
                    "Task Queue",
                    &lavs_task_queue::msg::InstantiateMsg {
                        requestor: Requestor::OpenPayment(cosmwasm_std::coin(
                            100,
                            &client.querier.chain_config.gas_denom,
                        )),
                        timeout: TimeoutInfo::new(100),
                        verifier: verifier_addr.to_string(),
                    },
                    vec![],
                    None,
                )
                .await?;

            tracing::info!("Task Queue Tx Hash: {}", tx_resp.txhash);
            tracing::info!("Task Queue Address: {}", task_queue_addr);

            tracing::info!("");
            tracing::info!("---- All contracts instantiated successfully ----");
            tracing::info!("Mock Operators: {}", operators_addr);
            tracing::info!("Verifier Simple: {}", verifier_addr);
            tracing::info!("Task Queue: {}", task_queue_addr);
        }
    }

    Ok(())
}
