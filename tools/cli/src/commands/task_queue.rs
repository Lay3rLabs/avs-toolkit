use crate::{
    args::{TargetEnvironment, TaskQueueArgs},
    context::AppContext,
};
use anyhow::{bail, Context, Result};
use lavs_apis::id::TaskId;
use lavs_task_queue::msg::{ConfigResponse, CustomExecuteMsg, CustomQueryMsg, QueryMsg, Requestor};
use layer_climb::{prelude::*, proto::abci::TxResponse};

pub struct TaskQueue {
    pub ctx: AppContext,
    pub contract_addr: Address,
}

impl TaskQueue {
    pub async fn new(ctx: AppContext, task_queue_args: &TaskQueueArgs) -> Result<Self> {
        let addr_string = match task_queue_args.address.clone() {
            Some(x) => x,
            None => match ctx.args.target_env {
                TargetEnvironment::Local => std::env::var("LOCAL_TASK_QUEUE_ADDRESS")
                    .context("LOCAL_TASK_QUEUE_ADDRESS not found")?,
                TargetEnvironment::Testnet => std::env::var("TEST_TASK_QUEUE_ADDRESS")
                    .context("TEST_TASK_QUEUE_ADDRESS not found")?,
            },
        };

        let contract_addr = ctx.chain_config.parse_address(&addr_string)?;

        Ok(Self { ctx, contract_addr })
    }

    pub async fn add_task(
        &self,
        body: String,
        description: String,
        timeout: Option<u64>,
    ) -> Result<(TaskId, TxResponse)> {
        let payload = serde_json::from_str(&body).context("Failed to parse body into JSON")?;

        let contract_config: ConfigResponse = self
            .ctx
            .get_client()
            .await?
            .querier
            .contract_smart(
                &self.contract_addr,
                &QueryMsg::Custom(CustomQueryMsg::Config {}),
            )
            .await?;

        let payment = match contract_config.requestor {
            Requestor::OpenPayment(coin) => vec![new_coin(coin.amount, coin.denom)],
            Requestor::Fixed(addr) => {
                if addr != self.ctx.get_client().await?.addr.to_string() {
                    bail!("Only the requestor can pay for the task")
                }
                Vec::new()
            }
        };

        let tx_resp = self
            .ctx
            .get_client()
            .await?
            .contract_execute(
                &self.contract_addr,
                &CustomExecuteMsg::Create {
                    description,
                    timeout,
                    payload,
                },
                payment,
                None,
            )
            .await?;

        let task_id: TaskId = CosmosTxEvents::from(&tx_resp)
            .attr_first("wasm", "task_id")?
            .value()
            .parse()?;

        tracing::debug!("Task added with id: {task_id}");
        tracing::debug!("Tx hash: {}", tx_resp.txhash);

        Ok((task_id, tx_resp))
    }
}
