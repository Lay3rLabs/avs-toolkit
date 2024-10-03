use crate::{
    args::{TargetEnvironment, TaskQueueArgs},
    commands::{operator::OperatorQuerier, verifier::SimpleVerifierQuerier},
    context::AppContext,
};
use anyhow::{bail, Context, Result};
use cosmwasm_std::Order;
use lavs_apis::{
    id::TaskId,
    tasks::{CompletedTaskOverview, ListCompletedResponse, ListOpenResponse, OpenTaskOverview},
};
use lavs_task_queue::msg::{ConfigResponse, CustomExecuteMsg, CustomQueryMsg, QueryMsg, Requestor};
use layer_climb::{prelude::*, proto::abci::TxResponse};

use super::operator::Operator;

pub struct TaskQueue {
    pub _ctx: AppContext,
    pub contract_addr: Address,
    // tasks have the notion of a specific admin
    // so use this one client instead of the pool
    pub admin: SigningClient,
    pub querier: TaskQueueQuerier,
}

impl TaskQueue {
    pub async fn new(ctx: AppContext, task_queue_args: &TaskQueueArgs) -> Result<Self> {
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

        let admin = ctx.signing_client().await?;

        let querier = TaskQueueQuerier {
            ctx: ctx.clone(),
            contract_addr: contract_addr.clone(),
            querier: ctx.query_client().await?,
        };

        Ok(Self {
            _ctx: ctx,
            contract_addr,
            admin,
            querier,
        })
    }

    pub async fn add_task(
        &self,
        body: String,
        description: String,
        timeout: Option<u64>,
    ) -> Result<(TaskId, TxResponse)> {
        let payload = serde_json::from_str(&body).context("Failed to parse body into JSON")?;

        let contract_config = self.querier.config().await?;

        let payment = match contract_config.requestor {
            Requestor::OpenPayment(coin) => vec![new_coin(coin.amount, coin.denom)],
            Requestor::Fixed(addr) => {
                if addr != self.admin.addr.to_string() {
                    bail!("Only the requestor can pay for the task")
                }
                Vec::new()
            }
        };

        let tx_resp = self
            .admin
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

        tracing::info!("Task added with id: {task_id}");
        tracing::debug!("Tx hash: {}", tx_resp.txhash);

        Ok((task_id, tx_resp))
    }
}

pub struct TaskQueueQuerier {
    pub ctx: AppContext,
    pub contract_addr: Address,
    pub querier: QueryClient,
}

impl TaskQueueQuerier {
    pub async fn config(&self) -> Result<ConfigResponse> {
        self.querier
            .contract_smart(
                &self.contract_addr,
                &QueryMsg::Custom(CustomQueryMsg::Config {}),
            )
            .await
    }

    pub async fn task_queue_view(
        &self,
        start_after: Option<TaskId>,
        limit: Option<u32>,
    ) -> Result<TaskQueueView> {
        let contract_config: ConfigResponse = self.config().await?;

        let verifier_addr = self
            .ctx
            .chain_config()?
            .parse_address(&contract_config.verifier)?;

        let verifier_querier =
            SimpleVerifierQuerier::new(self.ctx.clone(), verifier_addr.clone()).await?;

        let operator_addr = verifier_querier.operator_addr().await?;

        let operator_querier =
            OperatorQuerier::new(self.ctx.clone(), operator_addr.clone()).await?;

        let operators = operator_querier.all_operators().await?;

        let tasks = self
            .tasks_view(start_after, limit, Order::Descending)
            .await?;

        Ok(TaskQueueView {
            verifier_addr,
            operator_addr,
            operators,
            tasks,
        })
    }

    pub async fn tasks_view(
        &self,
        start_after: Option<TaskId>,
        limit: Option<u32>,
        order: Order,
    ) -> Result<Vec<TaskView>> {
        let tasks_open: ListOpenResponse = self
            .querier
            .contract_smart(
                &self.contract_addr,
                &QueryMsg::Custom(CustomQueryMsg::ListOpen { start_after, limit }),
            )
            .await?;

        let tasks_completed: ListCompletedResponse = self
            .querier
            .contract_smart(
                &self.contract_addr,
                &QueryMsg::Custom(CustomQueryMsg::ListCompleted { start_after, limit }),
            )
            .await?;

        let mut all_tasks =
            Vec::with_capacity(tasks_open.tasks.len() + tasks_completed.tasks.len());

        for task in tasks_open.tasks {
            all_tasks.push(TaskView::Open(task));
        }

        for task in tasks_completed.tasks {
            all_tasks.push(TaskView::Completed(task));
        }

        all_tasks.sort_by(|a, b| match order {
            Order::Ascending => a.id().cmp(&b.id()),
            Order::Descending => b.id().cmp(&a.id()),
        });

        Ok(all_tasks)
    }
}

pub struct TaskQueueView {
    pub verifier_addr: Address,
    pub operator_addr: Address,
    pub operators: Vec<Operator>,
    pub tasks: Vec<TaskView>,
}

impl TaskQueueView {
    pub fn report(&self, log: impl Fn(&str)) -> Result<()> {
        log(&format!("Verifier: {}", self.verifier_addr));
        log(&format!("Operator: {}", self.operator_addr));

        log("\nOperators:");
        for operator in &self.operators {
            log(&format!("  - {}: {}", operator.address, operator.power));
        }

        log("\nTasks:");

        for task in &self.tasks {
            let data_json_string = task.data_json_string()?;

            match task {
                TaskView::Open(task) => {
                    log(&format!("  - Open Task: {}", task.id));
                    log(&format!("    Expires: {}", task.expires));
                    log(&format!("    Payload: {}", data_json_string));
                }
                TaskView::Completed(task) => {
                    log(&format!("  - Completed Task: {}", task.id));
                    log(&format!("    Completed: {}", task.completed));
                    log(&format!("    Result: {}", data_json_string));
                }
            }
        }

        Ok(())
    }
}

pub enum TaskView {
    Open(OpenTaskOverview),
    Completed(CompletedTaskOverview),
}

impl TaskView {
    pub fn id(&self) -> TaskId {
        match self {
            TaskView::Open(task) => task.id,
            TaskView::Completed(task) => task.id,
        }
    }

    pub fn data_json_string(&self) -> Result<String> {
        serde_json::to_string_pretty(match self {
            TaskView::Open(task) => &task.payload,
            TaskView::Completed(task) => &task.result,
        })
        .map_err(|e| anyhow::anyhow!("Failed to serialize payload: {}", e))
    }
}
