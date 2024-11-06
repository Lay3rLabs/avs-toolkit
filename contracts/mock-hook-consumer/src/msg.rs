use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_orch::{ExecuteFns, QueryFns};
use lavs_apis::{
    id::TaskId,
    interfaces::task_hooks::{TaskHookExecuteMsg, TaskHookType},
};

#[cw_serde]
#[derive(ExecuteFns)]
pub enum ExecuteMsg {
    RegisterHook {
        task_id: TaskId,
        hook_type: TaskHookType,
    },
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
