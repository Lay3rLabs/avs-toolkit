use cosmwasm_schema::cw_serde;

// This pulls in the queries
pub use lavs_apis::interfaces::voting::*;

#[cw_serde]
pub struct InstantiateMsg {
    pub operators: Vec<(String, u64)>,
}

#[cw_serde]
#[derive(cw_orch::ExecuteFns)]
#[cw_orch(disable_fields_sorting)]
pub enum ExecuteMsg {}
