use cosmwasm_schema::{cw_serde, QueryResponses};
/// These are queries and messages can be sent to any operator group implementation
/// by other contracts. This specifies the required APIs for on-chain interactions.
/// This must be a subset of any of the implementation.
/// It is based on the DAO DAO APIs, so those existing contracts will be compatible.
use cosmwasm_std::Uint128;
// FIXME: do we need to derive cw_orch? This is not used by the contracts calling,
// And the external callers likely use the full APIs.
// TODO: try embedding these (enum with serde(untagged)) in the main types of the implementing contracts
use cw_orch::QueryFns;

/// Based on what is generated by #[voting_query] macro in dao dao
#[cw_serde]
#[derive(QueryFns)]
#[cw_orch(disable_fields_sorting)]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VotingPowerResponse)]
    VotingPowerAtHeight {
        address: String,
        height: Option<u64>,
    },
    #[returns(TotalPowerResponse)]
    TotalPowerAtHeight { height: Option<u64> },
    // TODO: move this to custom query...
    #[returns(AllVotersResponse)]
    AllVoters {},
}

/// Copy of dao_interface::voting::VotingPowerAtHeightResponse
#[cw_serde]
pub struct VotingPowerResponse {
    pub power: Uint128,
    pub height: u64,
}

/// Copy of dao_interface::voting::TotalPowerAtHeightResponse
#[cw_serde]
pub struct TotalPowerResponse {
    pub power: Uint128,
    pub height: u64,
}

#[cw_serde]
pub struct AllVotersResponse {
    pub voters: Vec<VoterInfo>,
}

#[cw_serde]
pub struct VoterInfo {
    pub power: Uint128,
    pub address: String,
}
