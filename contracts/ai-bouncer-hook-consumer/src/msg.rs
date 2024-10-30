use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_orch::{ExecuteFns, QueryFns};
use lavs_apis::interfaces::task_hooks::TaskHookExecuteMsg;

#[cw_serde]
pub struct InstantiateMsg {
    /// the DAO address
    pub dao: String,
    /// the cw4-group address
    pub cw4_group: String,
    /// the task queue address
    pub task_queue: String,
    /// the cw4-group weight to assign to new members
    pub new_member_weight: u64,
}

#[cw_serde]
#[derive(ExecuteFns)]
pub enum ExecuteMsg {
    /// set up the contract by registering it with the task queue. this is only
    /// necessary after unregistering, since registration automatically happens
    /// on instantiate. only the DAO can do this.
    Register {},
    /// teardown the contract by unregistering it with the task queue and
    /// changing the cw4-group contract admin back to the DAO. only the DAO can
    /// do this.
    Unregister {},
    /// update the DAO address. only the DAO can do this.
    UpdateDao { dao: String },
    /// update the cw4-group address. only the DAO can do this.
    UpdateCw4Group { cw4_group: String },
    /// update the task queue address. only the DAO can do this.
    UpdateTaskQueue { task_queue: String },
    /// update the new member weight. only the DAO can do this.
    UpdateNewMemberWeight { weight: u64 },
    /// execute messages on the cw4-group contract, since this contract is the
    /// admin. only the DAO can do this.
    Cw4Group(cw4_group::msg::ExecuteMsg),
    /// handle a task hook. only the task queue can call this.
    #[serde(untagged)]
    TaskHook(TaskHookExecuteMsg),
}

#[cw_serde]
#[derive(QueryResponses, QueryFns)]
pub enum QueryMsg {
    /// returns the decision made by the AI bouncer for a given user, if any.
    #[returns(Option<bool>)]
    Decision { address: String },
    /// returns the DAO contract address.
    #[returns(String)]
    Dao {},
    /// returns the cw4-group contract address.
    #[returns(String)]
    Cw4Group {},
    /// returns the task queue contract address.
    #[returns(String)]
    TaskQueue {},
    /// returns the new member weight.
    #[returns(u64)]
    NewMemberWeight {},
}

#[cw_serde]
pub enum TaskOutput {
    Success(TaskOutputSuccess),
    Error(String),
}

#[cw_serde]
#[serde(rename_all = "camelCase")]
pub struct TaskOutputSuccess {
    /// the session ID of the address being evaluated
    pub session_id: String,
    /// the address being evaluated
    pub address: String,
    /// the message ID being responded to
    pub message_id: u16,
    /// the response to the message
    pub response: String,
    /// the decision made by the AI bouncer, which will be present once
    /// finalized
    pub decision: Option<bool>,
}
