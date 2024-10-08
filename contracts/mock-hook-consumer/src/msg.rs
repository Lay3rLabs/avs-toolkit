use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw_orch::{ExecuteFns, QueryFns};
use lavs_apis::interfaces::task_hooks::TaskHookExecuteMsg;

#[cw_serde]
#[derive(ExecuteFns)]
pub enum ExecuteMsg {
    AddHooks {
        task_queue: String,
    },
    #[serde(untagged)]
    TaskHook(TaskHookExecuteMsg),
}

#[cw_serde]
#[derive(QueryResponses, QueryFns)]
pub enum QueryMsg {
    #[returns(Uint128)]
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
