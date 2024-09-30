use crate::context::AppContext;
use anyhow::{bail, Result};
use lavs_task_queue::msg::{Requestor, TimeoutInfo};
use std::path::PathBuf;
use tokio::try_join;

pub async fn deploy_contracts(ctx: AppContext, artifacts_path: PathBuf) -> Result<()> {
    tracing::debug!("Uploading contracts from {:?}", artifacts_path);

    let wasm_files = WasmFiles::read(artifacts_path.clone()).await?;

    let CodeIds {
        operators: operators_code_id,
        task_queue: task_queue_code_id,
        verifier_simple: verifier_code_id,
    } = CodeIds::upload(&ctx, wasm_files).await?;

    tracing::debug!("Contracts all uploaded successfully, instantiating...");

    let client = ctx.get_client().await?;

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

    tracing::debug!("Mock Operators Tx Hash: {}", tx_resp.txhash);
    tracing::debug!("Mock Operators Address: {}", operators_addr);

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

    tracing::debug!("Verifier Simple Tx Hash: {}", tx_resp.txhash);
    tracing::debug!("Verifier Simple Address: {}", verifier_addr);

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

    tracing::debug!("Task Queue Tx Hash: {}", tx_resp.txhash);
    tracing::debug!("Task Queue Address: {}", task_queue_addr);

    tracing::debug!("");
    tracing::debug!("---- All contracts instantiated successfully ----");
    tracing::debug!("Mock Operators: {}", operators_addr);
    tracing::debug!("Verifier Simple: {}", verifier_addr);
    tracing::debug!("Task Queue: {}", task_queue_addr);

    Ok(())
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

        let (operators_code_id, task_queue_code_id, verifier_code_id) = try_join!(
            {
                let ctx = ctx.clone();
                async move {
                    let client = ctx.get_client().await?;

                    tracing::debug!("Uploading Mock Operators from: {}", client.addr);
                    let (code_id, tx_resp) =
                        client.contract_upload_file(operators_wasm, None).await?;
                    tracing::debug!("Mock Operators Tx Hash: {}", tx_resp.txhash);
                    tracing::debug!("Mock Operators Code ID: {}", code_id);
                    anyhow::Ok(code_id)
                }
            },
            {
                let ctx = ctx.clone();
                async move {
                    let client = ctx.get_client().await?;

                    tracing::debug!("Uploading Task Queue from: {}", client.addr);
                    let (code_id, tx_resp) =
                        client.contract_upload_file(task_queue_wasm, None).await?;
                    tracing::debug!("Task Queue Tx Hash: {}", tx_resp.txhash);
                    tracing::debug!("Task Queue Code ID: {}", code_id);
                    anyhow::Ok(code_id)
                }
            },
            {
                let ctx = ctx.clone();
                async move {
                    let client = ctx.get_client().await?;

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
