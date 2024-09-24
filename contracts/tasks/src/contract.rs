#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use lavs_apis::interfaces::tasks as interface;
use lavs_apis::tasks::{CustomExecuteMsg, CustomQueryMsg, TaskQueryMsg};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::msg::{RequestType, ResponseType, Status};
use crate::state::{Config, Task, CONFIG, TASKS};

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
    let config = Config::validate(deps.as_ref(), msg)?;
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
        ExecuteMsg::Api(api) => match api {
            interface::TaskExecuteMsg::Complete { task_id, response } => {
                execute::complete(deps, env, info, task_id, response)
            }
        },
        ExecuteMsg::Custom(custom) => match custom {
            CustomExecuteMsg::Create {
                description,
                timeout,
                payload,
            } => execute::create(deps, env, info, description, timeout, payload),
            CustomExecuteMsg::Timeout { task_id } => execute::timeout(deps, env, info, task_id),
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Api(api) => match api {
            TaskQueryMsg::TaskStatus { id } => {
                Ok(to_json_binary(&query::task_status(deps, env, id)?)?)
            }
        },
        QueryMsg::Custom(custom) => match custom {
            CustomQueryMsg::Task { id } => Ok(to_json_binary(&query::task(deps, env, id)?)?),
            CustomQueryMsg::ListOpen {} => Ok(to_json_binary(&query::list_open(deps, env)?)?),
            CustomQueryMsg::ListCompleted {} => {
                Ok(to_json_binary(&query::list_completed(deps, env)?)?)
            }
            CustomQueryMsg::Config {} => Ok(to_json_binary(&query::config(deps, env)?)?),
        },
    }
}

mod execute {
    use cw_utils::nonpayable;

    use crate::state::Timing;

    use super::*;

    pub fn create(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        description: String,
        timeout: Option<u64>,
        payload: RequestType,
    ) -> Result<Response, ContractError> {
        let mut config = CONFIG.load(deps.storage)?;
        let timeout = config.timeout.check_timeout(timeout)?;
        config.requestor.check_requestor(&info)?;

        let timing = Timing::new(&env, timeout);
        let status = Status::new();
        let task = Task {
            description,
            status,
            timing,
            payload,
            result: None,
        };
        let task_id = config.next_id;
        TASKS.save(deps.storage, task_id, &task)?;
        config.next_id += 1;
        CONFIG.save(deps.storage, &config)?;

        let res = Response::new()
            .add_attribute("action", "create")
            .add_attribute("task_id", task_id.to_string());
        Ok(res)
    }

    pub fn complete(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        task_id: u64,
        response: ResponseType,
    ) -> Result<Response, ContractError> {
        nonpayable(&info)?;

        let config = CONFIG.load(deps.storage)?;
        if info.sender != config.verifier {
            return Err(ContractError::Unauthorized {});
        }

        // ensures it is open and not expired, then store response
        let mut task = TASKS.load(deps.storage, task_id)?;
        task.complete(&env, response)?;
        TASKS.save(deps.storage, task_id, &task)?;

        let res = Response::new()
            .add_attribute("action", "completed")
            .add_attribute("task_id", task_id.to_string());
        Ok(res)
    }

    pub fn timeout(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        task_id: u64,
    ) -> Result<Response, ContractError> {
        nonpayable(&info)?;

        // ensures it is open and expired
        let mut task = TASKS.load(deps.storage, task_id)?;
        task.expire(&env)?;
        TASKS.save(deps.storage, task_id, &task)?;

        let res = Response::new()
            .add_attribute("action", "expired")
            .add_attribute("task_id", task_id.to_string());
        Ok(res)
    }
}

mod query {
    use lavs_apis::tasks::{ConfigResponse, TaskStatus};

    use crate::msg::{
        CompletedTaskOverview, ListCompletedResponse, ListOpenResponse, OpenTaskOverview,
        TaskResponse, TaskStatusResponse,
    };

    use super::*;

    pub fn task(deps: Deps, _env: Env, id: u64) -> Result<TaskResponse, ContractError> {
        let task = TASKS.load(deps.storage, id)?;
        let r = TaskResponse {
            id,
            description: task.description,
            status: task.status,
            payload: task.payload,
            result: task.result,
        };
        Ok(r)
    }

    pub fn task_status(deps: Deps, env: Env, id: u64) -> Result<TaskStatusResponse, ContractError> {
        let task = TASKS.load(deps.storage, id)?;
        let status = match task.status {
            Status::Open {} if !task.timing.is_expired(&env) => TaskStatus::Open,
            Status::Expired {} | Status::Open {} => TaskStatus::Expired,
            Status::Completed { .. } => TaskStatus::Completed,
        };
        let r = TaskStatusResponse {
            id,
            status,
            created_height: task.timing.created_height,
            created_time: task.timing.created_at,
            expires_time: task.timing.expires_at,
        };
        Ok(r)
    }

    pub fn config(deps: Deps, _env: Env) -> Result<ConfigResponse, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        let r = ConfigResponse {
            requestor: config.requestor.into(),
            timeout: config.timeout.into(),
            verifier: config.verifier.into_string(),
        };
        Ok(r)
    }

    pub fn list_open(deps: Deps, env: Env) -> Result<ListOpenResponse, ContractError> {
        // TODO: proper implementation here, this is just for minimal test code
        let mut open = TASKS
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .filter_map(|r| match r {
                Ok((
                    id,
                    Task {
                        payload,
                        status: Status::Open {},
                        timing,
                        ..
                    },
                )) if timing.expires_at > env.block.time.seconds() => Some(Ok(OpenTaskOverview {
                    id,
                    expires: timing.expires_at,
                    payload,
                })),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<Vec<_>, _>>()?;

        open.sort_by_key(|t| t.expires);
        Ok(ListOpenResponse { tasks: open })
    }

    pub fn list_completed(deps: Deps, _env: Env) -> Result<ListCompletedResponse, ContractError> {
        let mut completed = TASKS
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .filter_map(|r| match r {
                Ok((
                    id,
                    Task {
                        result,
                        status: Status::Completed { completed, .. },
                        ..
                    },
                )) => Some(Ok(CompletedTaskOverview {
                    id,
                    completed,
                    result: result.unwrap(),
                })),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<Vec<_>, _>>()?;

        completed.sort_by_key(|t| t.completed);
        completed.reverse();
        Ok(ListCompletedResponse { tasks: completed })
    }
}

#[cfg(test)]
mod tests {}