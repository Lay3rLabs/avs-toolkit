use crate::{args::DeployTaskRequestor, config::load_wasmatic_addresses, context::AppContext};
use anyhow::{anyhow, bail, Result};
use lavs_task_queue::msg::{Requestor, TimeoutInfo};
use layer_climb::prelude::*;
use std::path::PathBuf;
use tokio::try_join;

#[derive(Debug)]
pub struct DeployContractArgs {
    artifacts_path: PathBuf,
    operators: Vec<lavs_mock_operators::msg::InstantiateOperator>,
    requestor: Requestor,
    task_timeout: TimeoutInfo,
    required_voting_percentage: u32,
}

impl DeployContractArgs {
    pub async fn parse(
        ctx: &AppContext,
        artifacts_path: PathBuf,
        task_timeout_seconds: u64,
        required_voting_percentage: u32,
        operators: Vec<String>,
        requestor: DeployTaskRequestor,
    ) -> Result<Self> {
        if operators.is_empty() {
            bail!("At least one operator must be specified");
        }

        let mut instantiate_operators = vec![];
        for s in operators.into_iter() {
            let mut parts = s.split(':');
            let addr_str = parts.next().unwrap().to_string();
            match addr_str.as_str() {
                "wasmatic" => {
                    let chain_info = ctx.chain_info()?;
                    let chain_config = &chain_info.chain;
                    let wasmatic_addresses: Vec<Address> =
                        load_wasmatic_addresses(&chain_info.wasmatic.endpoints)
                            .await?
                            .into_iter()
                            .map(|addr| {
                                Address::try_from_value(&addr, &chain_config.address_kind).unwrap()
                            })
                            .collect();
                    for addr in wasmatic_addresses {
                        let voting_power = parts.next().unwrap_or("1").parse()?;
                        instantiate_operators.push(
                            lavs_mock_operators::msg::InstantiateOperator::new(
                                addr.to_string(),
                                voting_power,
                            ),
                        );
                    }
                }
                _ => {
                    let addr = ctx.chain_config()?.parse_address(&addr_str)?;
                    let voting_power = parts.next().unwrap_or("1").parse()?;
                    instantiate_operators.push(lavs_mock_operators::msg::InstantiateOperator::new(
                        addr.to_string(),
                        voting_power,
                    ));
                }
            }
        }

        let requestor = match requestor {
            DeployTaskRequestor::Deployer => {
                Requestor::Fixed(ctx.signing_client().await?.addr.to_string())
            }
            DeployTaskRequestor::Fixed(s) => {
                Requestor::Fixed(ctx.chain_config()?.parse_address(&s)?.to_string())
            }
            DeployTaskRequestor::Payment { amount, denom } => {
                Requestor::OpenPayment(cosmwasm_std::coin(
                    amount,
                    denom.unwrap_or(ctx.chain_config()?.gas_denom.clone()),
                ))
            }
        };

        let task_timeout = TimeoutInfo::new(task_timeout_seconds);

        Ok(Self {
            artifacts_path,
            operators: instantiate_operators,
            requestor,
            task_timeout,
            required_voting_percentage,
        })
    }
}

pub async fn deploy_contracts(
    ctx: AppContext,
    args: DeployContractArgs,
) -> Result<DeployContractAddrs> {
    tracing::debug!("Deploying contracts with args: {:#?}", args);

    let DeployContractArgs {
        artifacts_path,
        operators,
        requestor,
        task_timeout,
        required_voting_percentage,
    } = args;

    let wasm_files = WasmFiles::read(artifacts_path.clone()).await?;

    let CodeIds {
        operators: operators_code_id,
        task_queue: task_queue_code_id,
        verifier_simple: verifier_code_id,
    } = CodeIds::upload(&ctx, wasm_files).await?;

    tracing::debug!("Contracts all uploaded successfully, instantiating...");

    let client = ctx.signing_client().await?;

    let (operators_addr, tx_resp) = client
        .contract_instantiate(
            client.addr.clone(),
            operators_code_id,
            "Mock Operators",
            &lavs_mock_operators::msg::InstantiateMsg { operators },
            vec![],
            None,
        )
        .await?;

    tracing::debug!("Mock Operators Tx Hash: {}", tx_resp.txhash);
    tracing::debug!("Mock Operators Address: {}", operators_addr);

    let (verifier_addr, tx_resp) = client
        .contract_instantiate(
            client.addr.clone(),
            verifier_code_id,
            "Verifier Simple",
            &lavs_verifier_simple::msg::InstantiateMsg {
                operator_contract: operators_addr.to_string(),
                required_percentage: required_voting_percentage,
            },
            vec![],
            None,
        )
        .await?;

    tracing::debug!("Verifier Simple Tx Hash: {}", tx_resp.txhash);
    tracing::debug!("Verifier Simple Address: {}", verifier_addr);

    let (task_queue_addr, tx_resp) = client
        .contract_instantiate(
            client.addr.clone(),
            task_queue_code_id,
            "Task Queue",
            &lavs_task_queue::msg::InstantiateMsg {
                requestor,
                timeout: task_timeout,
                verifier: verifier_addr.to_string(),
            },
            vec![],
            None,
        )
        .await?;

    tracing::debug!("Task Queue Tx Hash: {}", tx_resp.txhash);
    tracing::debug!("Task Queue Address: {}", task_queue_addr);

    Ok(DeployContractAddrs {
        operators: operators_addr,
        task_queue: task_queue_addr,
        verifier_simple: verifier_addr,
    })
}

pub struct DeployContractAddrs {
    pub operators: Address,
    pub task_queue: Address,
    pub verifier_simple: Address,
}

struct WasmFiles {
    operators: Vec<u8>,
    task_queue: Vec<u8>,
    verifier_simple: Vec<u8>,
}

impl WasmFiles {
    pub async fn read(artifacts_path: PathBuf) -> Result<Self> {
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

        let (operators, task_queue, verifier_simple) = try_join!(
            tokio::fs::read(operators_path),
            tokio::fs::read(task_queue_path),
            tokio::fs::read(verifier_simple_path)
        )?;

        Ok(Self {
            operators,
            task_queue,
            verifier_simple,
        })
    }
}

struct CodeIds {
    operators: u64,
    task_queue: u64,
    verifier_simple: u64,
}

impl CodeIds {
    pub async fn upload(ctx: &AppContext, files: WasmFiles) -> Result<Self> {
        let WasmFiles {
            operators: operators_wasm,
            task_queue: task_queue_wasm,
            verifier_simple: verifier_simple_wasm,
        } = files;

        let client_pool = ctx.create_client_pool().await?;

        let (operators_code_id, task_queue_code_id, verifier_code_id) = try_join!(
            {
                let client_pool = client_pool.clone();
                async move {
                    let client = client_pool.get().await.map_err(|e| anyhow!("{e:?}"))?;

                    tracing::debug!("Uploading Mock Operators from: {}", client.addr);
                    let (code_id, tx_resp) =
                        client.contract_upload_file(operators_wasm, None).await?;
                    tracing::debug!("Mock Operators Tx Hash: {}", tx_resp.txhash);
                    tracing::debug!("Mock Operators Code ID: {}", code_id);
                    anyhow::Ok(code_id)
                }
            },
            {
                let client_pool = client_pool.clone();
                async move {
                    let client = client_pool.get().await.map_err(|e| anyhow!("{e:?}"))?;

                    tracing::debug!("Uploading Task Queue from: {}", client.addr);
                    let (code_id, tx_resp) =
                        client.contract_upload_file(task_queue_wasm, None).await?;
                    tracing::debug!("Task Queue Tx Hash: {}", tx_resp.txhash);
                    tracing::debug!("Task Queue Code ID: {}", code_id);
                    anyhow::Ok(code_id)
                }
            },
            {
                let client_pool = client_pool.clone();
                async move {
                    let client = client_pool.get().await.map_err(|e| anyhow!("{e:?}"))?;

                    tracing::debug!("Uploading Simple Verifier from: {}", client.addr);
                    let (code_id, tx_resp) = client
                        .contract_upload_file(verifier_simple_wasm, None)
                        .await?;
                    tracing::debug!("Simple Verifier Tx Hash: {}", tx_resp.txhash);
                    tracing::debug!("Simple Verifier Code ID: {}", code_id);
                    anyhow::Ok(code_id)
                }
            }
        )?;

        Ok(Self {
            operators: operators_code_id,
            task_queue: task_queue_code_id,
            verifier_simple: verifier_code_id,
        })
    }
}
