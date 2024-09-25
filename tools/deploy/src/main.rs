mod args;

use anyhow::Result;
use args::{Args, Command};
use clap::Parser;
use cw_orch::{
    daemon::{DaemonBase, Wallet},
    prelude::*,
};
use lavs_orch::daemon::slay3r_connect;
use lavs_task_queue::msg::{Requestor, TimeoutInfo};

fn main() -> Result<()> {
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

    // this is all necessary to play nicely with cw-orch
    // will be removed when we can have a proper async client
    let rt = tokio::runtime::Runtime::new().unwrap();
    let args = rt.block_on(async move { args::Args::new(args).await })?;

    let daemon = slay3r_connect(args.chain_kind.into(), Some(rt.handle()))?;

    inner_main(args, daemon)?;
    Ok(())
}

fn inner_main(args: Args, daemon: DaemonBase<Wallet>) -> Result<()> {
    match args.command {
        Command::DeployAvsContracts {} => {
            tracing::info!("connecting...");
            tracing::info!("{:?}", daemon.chain_info());

            let mock_operators = lavs_mock_operators::interface::Contract::new(daemon.clone());
            mock_operators.upload()?;
            let mock_operators_addr = mock_operators
                .instantiate(
                    &lavs_mock_operators::msg::InstantiateMsg {
                        operators: vec![lavs_mock_operators::msg::InstantiateOperator::new(
                            daemon.sender_addr().to_string(),
                            1,
                        )],
                    },
                    None,
                    &[],
                )?
                .instantiated_contract_address()?;

            let verifier = lavs_verifier_simple::interface::Contract::new(daemon.clone());
            verifier.upload()?;
            let verifier_addr = verifier
                .instantiate(
                    &lavs_verifier_simple::msg::InstantiateMsg {
                        operator_contract: mock_operators_addr.to_string(),
                        required_percentage: 1,
                    },
                    None,
                    &[],
                )?
                .instantiated_contract_address()?;

            let task_queue = lavs_task_queue::interface::Contract::new(daemon.clone());
            task_queue.upload()?;
            let task_queue_addr = task_queue
                .instantiate(
                    &lavs_task_queue::msg::InstantiateMsg {
                        requestor: Requestor::OpenPayment(Coin::new(100u128, "uslay")),
                        timeout: TimeoutInfo::new(100),
                        verifier: verifier_addr.to_string(),
                    },
                    None,
                    &[],
                )?
                .instantiated_contract_address()?;

            tracing::info!("---- All contracts deployed ----");
            tracing::info!("operators addr: {}", mock_operators_addr);
            tracing::info!("verifier addr: {}", verifier_addr);
            tracing::info!("task_queue addr: {}", task_queue_addr);
        }
    }

    Ok(())
}
