#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, InstantiateOperator, QueryMsg};
use crate::state::{Config, OpInfo, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let mut total_power = Uint128::zero();
    let operators = msg
        .operators
        .into_iter()
        .map(|InstantiateOperator { addr, voting_power }| {
            let op = deps.api.addr_validate(&addr)?;
            let power = Uint128::from(voting_power);
            total_power += power;
            Ok(OpInfo { op, power })
        })
        .collect::<StdResult<Vec<_>>>()?;
    let config = Config {
        operators,
        total_power,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VotingPowerAtHeight { address, height } => {
            to_json_binary(&query::voting_power(deps, env, address, height)?)
        }
        QueryMsg::TotalPowerAtHeight { height } => {
            to_json_binary(&query::total_power(deps, env, height)?)
        }
        QueryMsg::AllVoters {} => to_json_binary(&query::all_voters(deps, env)?),
    }
}

// Although the queries take a height parameter, they don't use it to query historical data.
// This doesn't matter here since we're just mocking the operators, and they don't change.
mod query {
    use super::*;

    use lavs_apis::interfaces::voting::{
        AllVotersResponse, TotalPowerResponse, VoterInfo, VotingPowerResponse,
    };

    pub fn voting_power(
        deps: Deps,
        env: Env,
        address: String,
        height: Option<u64>,
    ) -> StdResult<VotingPowerResponse> {
        let height = height.unwrap_or(env.block.height);
        let addr = deps.api.addr_validate(&address)?;
        let config = CONFIG.load(deps.storage)?;
        let op = config.operators.iter().find(|op| op.op == addr);
        let power = op.map(|op| op.power).unwrap_or_default();
        Ok(VotingPowerResponse { power, height })
    }

    pub fn total_power(deps: Deps, env: Env, height: Option<u64>) -> StdResult<TotalPowerResponse> {
        let height = height.unwrap_or(env.block.height);
        let power = CONFIG.load(deps.storage)?.total_power;
        let res = TotalPowerResponse { power, height };
        Ok(res)
    }

    pub fn all_voters(deps: Deps, _env: Env) -> StdResult<AllVotersResponse> {
        let config = CONFIG.load(deps.storage)?;
        let voters = config
            .operators
            .into_iter()
            .map(|op| VoterInfo {
                power: op.power,
                address: op.op.into_string(),
            })
            .collect();
        Ok(AllVotersResponse { voters })
    }
}

#[cfg(test)]
mod tests {}
