use crate::context::AppContext;
use anyhow::{anyhow, bail, Result};
use avs_toolkit_shared::deploy::CodeIds;
use std::path::PathBuf;
use tokio::try_join;

pub struct WasmFiles {
    pub mock_operators: Vec<u8>,
    pub task_queue: Vec<u8>,
    pub verifier_simple: Vec<u8>,
    pub verifier_oracle: Vec<u8>,
}

impl WasmFiles {
    pub async fn read(artifacts_path: PathBuf) -> Result<Self> {
        let mock_operators_path = artifacts_path.join("lavs_mock_operators.wasm");
        let task_queue_path = artifacts_path.join("lavs_task_queue.wasm");
        let verifier_simple_path = artifacts_path.join("lavs_verifier_simple.wasm");
        let verifier_oracle_path = artifacts_path.join("lavs_oracle_verifier.wasm");

        if !mock_operators_path.exists() {
            bail!(
                "Mock Operators contract not found at {} (try running optimize.sh)",
                mock_operators_path.display()
            );
        }
        if !task_queue_path.exists() {
            bail!(
                "Task Queue contract not found at {} (try running optimize.sh)",
                task_queue_path.display()
            );
        }
        if !verifier_simple_path.exists() {
            bail!(
                "Verifier Simple contract not found at {} (try running optimize.sh)",
                verifier_simple_path.display()
            );
        }
        if !verifier_oracle_path.exists() {
            bail!(
                "Verifier Oracle contract not found at {} (try running optimize.sh)",
                verifier_oracle_path.display()
            );
        }

        let (mock_operators, task_queue, verifier_simple, verifier_oracle) = try_join!(
            tokio::fs::read(mock_operators_path),
            tokio::fs::read(task_queue_path),
            tokio::fs::read(verifier_simple_path),
            tokio::fs::read(verifier_oracle_path),
        )?;

        Ok(Self {
            mock_operators,
            task_queue,
            verifier_simple,
            verifier_oracle,
        })
    }
}

pub async fn upload_contracts(ctx: &AppContext, files: WasmFiles) -> Result<CodeIds> {
    let WasmFiles {
        mock_operators: mock_operators_wasm,
        task_queue: task_queue_wasm,
        verifier_simple: verifier_simple_wasm,
        verifier_oracle: verifier_oracle_wasm,
    } = files;

    let client_pool = ctx.create_client_pool().await?;

    let (
        mock_operators_code_id,
        task_queue_code_id,
        verifier_simple_code_id,
        verifier_oracle_code_id,
    ) = try_join!(
        {
            let client_pool = client_pool.clone();
            async move {
                let client = client_pool.get().await.map_err(|e| anyhow!("{e:?}"))?;

                tracing::debug!("Uploading Mock Operators from: {}", client.addr);
                let (code_id, tx_resp) = client
                    .contract_upload_file(mock_operators_wasm, None)
                    .await?;
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
                let (code_id, tx_resp) = client.contract_upload_file(task_queue_wasm, None).await?;
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
        },
        {
            let client_pool = client_pool.clone();
            async move {
                let client = client_pool.get().await.map_err(|e| anyhow!("{e:?}"))?;

                tracing::debug!("Uploading Oracle Verifier from: {}", client.addr);
                let (code_id, tx_resp) = client
                    .contract_upload_file(verifier_oracle_wasm, None)
                    .await?;
                tracing::debug!("Oracle Verifier Tx Hash: {}", tx_resp.txhash);
                tracing::debug!("Oracle Verifier Code ID: {}", code_id);
                anyhow::Ok(code_id)
            }
        }
    )?;

    Ok(CodeIds {
        mock_operators: mock_operators_code_id,
        task_queue: task_queue_code_id,
        verifier_simple: verifier_simple_code_id,
        verifier_oracle: verifier_oracle_code_id,
    })
}
