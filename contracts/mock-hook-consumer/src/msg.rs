use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_orch::{ExecuteFns, QueryFns};
use lavs_apis::interfaces::task_hooks::TaskHookExecuteMsg;

#[cw_serde]
#[derive(ExecuteFns)]
pub enum ExecuteMsg {
    #[serde(untagged)]
    TaskHook(TaskHookExecuteMsg),
}

#[cw_serde]
#[derive(QueryResponses, QueryFns)]
pub enum QueryMsg {
    #[returns(u64)]
    CreatedCount {},
}

#[cw_serde]
pub struct TaskRequestData {
    pub x: u64,
}

#[cw_serde]
pub struct TaskResponseData {
    pub y: u64,
}
