use cosmwasm_schema::cw_serde;
use lavs_apis::interfaces::task_hooks::TaskHookExecuteMsg;

#[cw_serde]
pub enum ExecuteMsg {
    TaskHook(TaskHookExecuteMsg),
}

#[cw_serde]
pub struct TaskRequestData {
    pub x: u64,
}

#[cw_serde]
pub struct TaskResponseData {
    pub y: u64,
}
