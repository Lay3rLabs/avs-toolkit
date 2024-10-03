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
            CustomQueryMsg::ListOpen { start_after, limit } => Ok(to_json_binary(
                &query::list_open(deps, env, start_after, limit)?,
            )?),
            CustomQueryMsg::ListCompleted { start_after, limit } => Ok(to_json_binary(
                &query::list_completed(deps, env, start_after, limit)?,
            )?),
            CustomQueryMsg::Config {} => Ok(to_json_binary(&query::config(deps, env)?)?),
        },
    }
}

mod execute {
    use cw_utils::nonpayable;
    use lavs_apis::id::TaskId;

    use crate::state::{check_timeout, Timing};

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
        let timeout = check_timeout(&config.timeout, timeout)?;
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
        config.next_id = TaskId::new(task_id.u64() + 1);
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
        task_id: TaskId,
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
        task_id: TaskId,
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
    use cw_storage_plus::Bound;
    use lavs_apis::{id::TaskId, tasks::ConfigResponse};

    use crate::msg::{
        CompletedTaskOverview, ListCompletedResponse, ListOpenResponse, OpenTaskOverview,
        TaskResponse, TaskStatusResponse,
    };

    use super::*;

    pub fn task(deps: Deps, env: Env, id: TaskId) -> Result<TaskResponse, ContractError> {
        let task = TASKS.load(deps.storage, id)?;
        let status = task.validate_status(&env);

        let r = TaskResponse {
            id,
            description: task.description,
            status,
            payload: task.payload,
            result: task.result,
        };
        Ok(r)
    }

    pub fn task_status(
        deps: Deps,
        env: Env,
        id: TaskId,
    ) -> Result<TaskStatusResponse, ContractError> {
        let task = TASKS.load(deps.storage, id)?;
        let status = task.validate_status(&env).into();

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
            timeout: config.timeout,
            verifier: config.verifier.into_string(),
        };
        Ok(r)
    }

    // TODO: There should probably be a max page limit, but it's left unbound to keep the API simple for now
    pub fn list_open(
        deps: Deps,
        env: Env,
        start_after: Option<TaskId>,
        limit: Option<u32>,
    ) -> Result<ListOpenResponse, ContractError> {
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);

        let open = TASKS
            .idx
            .status
            .prefix(Status::Open {}.as_str())
            .range(
                deps.storage,
                None,
                start_after.map(Bound::exclusive),
                cosmwasm_std::Order::Descending,
            )
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
            .take(limit)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ListOpenResponse { tasks: open })
    }

    pub fn list_completed(
        deps: Deps,
        _env: Env,
        start_after: Option<TaskId>,
        limit: Option<u32>,
    ) -> Result<ListCompletedResponse, ContractError> {
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);

        let completed = TASKS
            .idx
            .status
            .prefix(Status::Completed { completed: 0 }.as_str())
            .range(
                deps.storage,
                None,
                start_after.map(Bound::exclusive),
                cosmwasm_std::Order::Descending,
            )
            .filter_map(|r| match r {
                Ok((
                    id,
                    Task {
                        result,
                        status: Status::Completed { completed, .. },
                        ..
                    },
                )) => match result {
                    None => Some(Err(ContractError::MissingResultCompleted { id })),
                    Some(result) => Some(Ok(CompletedTaskOverview {
                        id,
                        completed,
                        result,
                    })),
                },
                Ok(_) => None,
                Err(e) => Some(Err(e.into())),
            })
            .take(limit)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ListCompletedResponse { tasks: completed })
    }
}

#[cfg(test)]
mod tests {}
