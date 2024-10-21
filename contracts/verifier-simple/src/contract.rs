#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::state::{Config, CONFIG};

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
    // validate the input data
    let operators = deps.api.addr_validate(&msg.operator_contract)?;
    let required_percentage = msg.required_percentage;
    if required_percentage > 100 || required_percentage == 0 {
        return Err(ContractError::InvalidPercentage);
    }

    // save config and cw2 metadata
    let config = Config {
        operators,
        required_percentage,
    };
    CONFIG.save(deps.storage, &config)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ExecutedTask {
            task_queue_contract,
            task_id,
            result,
        } => execute::executed_task(deps, env, info, task_queue_contract, task_id, result),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Ok(to_json_binary(&query::config(deps)?)?),
        QueryMsg::TaskInfo {
            task_contract,
            task_id,
        } => Ok(to_json_binary(&query::task_info(
            deps,
            env,
            task_contract,
            task_id,
        )?)?),
        QueryMsg::OperatorVote {
            task_contract,
            task_id,
            operator,
        } => Ok(to_json_binary(&query::operator_vote(
            deps,
            task_contract,
            task_id,
            operator,
        )?)?),
    }
}

mod execute {
    use super::*;

    use cosmwasm_std::{from_json, WasmMsg};

    use cw_utils::nonpayable;
    use lavs_apis::events::task_executed_event::TaskExecutedEvent;
    use lavs_apis::id::TaskId;
    use lavs_apis::interfaces::tasks::{ResponseType, TaskExecuteMsg, TaskStatus};
    use lavs_helpers::verifier::ensure_valid_vote;

    use crate::state::{record_vote, TASKS, VOTES};

    pub fn executed_task(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        task_queue_contract: String,
        task_id: TaskId,
        result: String,
    ) -> Result<Response, ContractError> {
        nonpayable(&info)?;

        // Ensure task is open and this operator can vote
        let task_queue = deps.api.addr_validate(&task_queue_contract)?;
        let operator = info.sender;

        // verify the result type upon submissions (parse it into expected ResponseType)
        let _: ResponseType = from_json(&result)?;

        let vote = VOTES.may_load(deps.storage, (&task_queue, task_id, &operator))?;
        let config = CONFIG.load(deps.storage)?;

        // Operator has not submitted a vote yet
        if vote.is_some() {
            return Err(ContractError::OperatorAlreadyVoted(operator.to_string()));
        }

        // Verify this operator is allowed to vote and has not voted yet, and do some initialization
        let (mut task_data, power) = match ensure_valid_vote(
            deps.branch(),
            &env,
            &task_queue,
            task_id,
            &operator,
            config.required_percentage,
            &config.operators,
        )? {
            Some(x) => x,
            None => return Ok(Response::default()),
        };

        // Update the vote and check the total power on this result, also recording the operators vote
        let tally = record_vote(
            deps.storage,
            &task_queue,
            task_id,
            &operator,
            &result,
            power,
        )?;

        let mut task_event = TaskExecutedEvent {
            task_id,
            task_queue: task_queue_contract.clone(),
            operator: operator.to_string(),
            completed: false,
        };

        let mut res = Response::new();

        // If there is enough power, let's submit it as completed
        // We add completed attribute to mark if this was the last one or not
        if tally >= task_data.power_required {
            // We need to update the status as completed
            task_data.status = TaskStatus::Completed;
            TASKS.save(deps.storage, (&task_queue, task_id), &task_data)?;

            // And submit the result to the task queue (after parsing it into relevant type)
            let response: ResponseType = from_json(&result)?;
            res = res.add_message(WasmMsg::Execute {
                contract_addr: task_queue_contract,
                msg: to_json_binary(&TaskExecuteMsg::Complete { task_id, response })?,
                funds: vec![],
            });
            task_event.completed = true;
        }

        res = res.add_event(task_event);

        Ok(res)
    }
}

mod query {
    use lavs_apis::id::TaskId;
    use lavs_apis::verifier_simple::{TaskStatus, TaskTally};

    use super::*;

    use crate::msg::{ConfigResponse, OperatorVoteInfoResponse, TaskInfoResponse};
    use crate::state::{OPTIONS, TASKS, VOTES};

    pub fn config(deps: Deps) -> StdResult<ConfigResponse> {
        let cfg = CONFIG.load(deps.storage)?;
        Ok(ConfigResponse {
            operator_contract: cfg.operators.to_string(),
            required_percentage: cfg.required_percentage,
        })
    }

    pub fn task_info(
        deps: Deps,
        env: Env,
        task_contract: String,
        task_id: TaskId,
    ) -> StdResult<Option<TaskInfoResponse>> {
        let task_contract = deps.api.addr_validate(&task_contract)?;
        let info = TASKS.may_load(deps.storage, (&task_contract, task_id))?;
        if let Some(i) = info {
            // Check current time and update the status if it expired
            let status = match i.status {
                TaskStatus::Open if i.is_expired(&env) => TaskStatus::Expired,
                x => x,
            };
            // Collect the running tallies on the options
            let tallies: Result<Vec<_>, _> = OPTIONS
                .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
                .map(|r| {
                    r.map(|((_, _, result), v)| TaskTally {
                        result,
                        power: v.power,
                    })
                })
                .collect();
            let res = TaskInfoResponse {
                status,
                power_needed: i.power_required,
                tallies: tallies?,
            };
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }

    pub fn operator_vote(
        deps: Deps,
        task_contract: String,
        task_id: TaskId,
        operator: String,
    ) -> StdResult<Option<OperatorVoteInfoResponse>> {
        let task_contract = deps.api.addr_validate(&task_contract)?;
        let operator = deps.api.addr_validate(&operator)?;
        let vote = VOTES
            .may_load(deps.storage, (&task_contract, task_id, &operator))?
            .map(|v| OperatorVoteInfoResponse {
                power: v.power,
                result: v.result,
            });
        Ok(vote)
    }
}

#[cfg(test)]
mod tests {}
