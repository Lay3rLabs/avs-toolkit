use anyhow::{bail, Result};
use cosmwasm_std::Order;
use lavs_apis::{
    events::{task_queue_events::TaskCreatedEvent, traits::TypedEvent as _},
    id::TaskId,
    tasks::{CompletedTaskOverview, ListCompletedResponse, ListOpenResponse, OpenTaskOverview},
    time::Duration,
};
use lavs_task_queue::msg::{ConfigResponse, CustomExecuteMsg, CustomQueryMsg, QueryMsg, Requestor};
use layer_climb::{prelude::*, proto::abci::TxResponse};

use crate::{operator::OperatorQuerier, verifier::SimpleVerifierQuerier};

use super::operator::Operator;

pub struct TaskQueue {
    pub contract_addr: Address,
    // tasks have the notion of a specific admin
    // so use this one client instead of the pool
    pub admin: SigningClient,
    pub querier: TaskQueueQuerier,
}

impl TaskQueue {
    pub async fn new(admin: SigningClient, contract_addr: Address) -> Self {
        let querier = TaskQueueQuerier {
            contract_addr: contract_addr.clone(),
            query_client: admin.querier.clone(),
        };

        Self {
            contract_addr,
            admin,
            querier,
        }
    }

    pub async fn add_task(
        &self,
        payload: serde_json::Value,
        description: String,
        timeout: Option<Duration>,
    ) -> Result<(TaskId, TxResponse)> {
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

        let event: cosmwasm_std::Event = CosmosTxEvents::from(&tx_resp)
            .event_first_by_type(TaskCreatedEvent::NAME)?
            .into();
        let event: TaskCreatedEvent = event.try_into()?;

        tracing::info!("Task added with id: {0}", event.task_id);
        tracing::debug!("Tx hash: {}", tx_resp.txhash);

        Ok((event.task_id, tx_resp))
    }
}

pub struct TaskQueueQuerier {
    pub contract_addr: Address,
    pub query_client: QueryClient,
}

impl TaskQueueQuerier {
    pub async fn config(&self) -> Result<ConfigResponse> {
        self.query_client
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
            .query_client
            .chain_config
            .parse_address(&contract_config.verifier)?;

        let verifier_querier =
            SimpleVerifierQuerier::new(self.query_client.clone(), verifier_addr.clone()).await?;

        let operator_addr = verifier_querier.operator_addr().await?;

        let operator_querier =
            OperatorQuerier::new(self.query_client.clone(), operator_addr.clone()).await?;

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
            .query_client
            .contract_smart(
                &self.contract_addr,
                &QueryMsg::Custom(CustomQueryMsg::ListOpen { start_after, limit }),
            )
            .await?;

        let tasks_completed: ListCompletedResponse = self
            .query_client
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
