#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, Event, MessageInfo, Reply, Response,
};
use cw2::set_contract_version;

use lavs_apis::interfaces::tasks as interface;
use lavs_apis::tasks::{CustomExecuteMsg, CustomQueryMsg, TaskQueryMsg};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::msg::{RequestType, ResponseType, Status};
use crate::state::{Config, Task, CONFIG, TASKS, TASK_HOOKS};

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const HOOK_ERROR_REPLY_ID: u64 = 0;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let owner = msg.owner.clone().unwrap_or(info.sender.to_string());
    let config = Config::validate(deps.as_ref(), msg)?;
    CONFIG.save(deps.storage, &config)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&owner))?;

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
            CustomExecuteMsg::AddHook {
                hook_type,
                receiver,
            } => execute::add_hook(deps, info, hook_type, receiver),
            CustomExecuteMsg::RemoveHook {
                hook_type,
                receiver,
            } => execute::remove_hook(deps, info, hook_type, receiver),
            CustomExecuteMsg::UpdateOwnership(action) => {
                let ownership =
                    cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;

                let event =
                    Event::new("update_ownership").add_attributes(ownership.into_attributes());

                Ok(Response::new().add_event(event))
            }
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
            CustomQueryMsg::List { start_after, limit } => Ok(to_json_binary(&query::list(
                deps,
                env,
                start_after,
                limit,
            )?)?),
            CustomQueryMsg::ListOpen { start_after, limit } => Ok(to_json_binary(
                &query::list_open(deps, env, start_after, limit)?,
            )?),
            CustomQueryMsg::ListCompleted { start_after, limit } => Ok(to_json_binary(
                &query::list_completed(deps, env, start_after, limit)?,
            )?),
            CustomQueryMsg::Config {} => Ok(to_json_binary(&query::config(deps, env)?)?),
            CustomQueryMsg::TaskHooks(hook_type) => {
                Ok(to_json_binary(&TASK_HOOKS.query_hooks(deps, hook_type)?)?)
            }
            CustomQueryMsg::Ownership {} => {
                Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?)
            }
        },
    }
}

mod execute {
    use cosmwasm_std::{BankMsg, SubMsg, WasmMsg};
    use cw_ownable::assert_owner;
    use cw_utils::nonpayable;
    use lavs_apis::{
        events::task_queue_events::{
            HookAddedEvent, HookRemovedEvent, TaskCompletedEvent, TaskCreatedEvent,
            TaskExpiredEvent,
        },
        id::TaskId,
        interfaces::task_hooks::{TaskHookExecuteMsg, TaskHookType},
        tasks::TaskResponse,
        time::Duration,
    };

    use crate::state::{check_timeout, RequestorConfig, TaskDeposit, Timing, TASK_DEPOSITS};

    use super::*;

    pub fn create(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        description: String,
        timeout: Option<Duration>,
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

        if let RequestorConfig::OpenPayment(coin) = config.requestor {
            TASK_DEPOSITS.save(
                deps.storage,
                task_id,
                &TaskDeposit {
                    addr: info.sender,
                    coin,
                },
            )?;
        }

        // Prepare hooks
        let hooks = TASK_HOOKS.created.prepare_hooks(deps.storage, |addr| {
            Ok(SubMsg::reply_on_error(
                WasmMsg::Execute {
                    contract_addr: addr.to_string(),
                    msg: to_json_binary(&TaskHookExecuteMsg::TaskCreatedHook(TaskResponse {
                        description: task.description.clone(),
                        status: task.status.clone(),
                        id: task_id,
                        payload: task.payload.clone(),
                        result: None,
                    }))?,
                    funds: vec![],
                },
                HOOK_ERROR_REPLY_ID,
            ))
        })?;

        let task_queue_event = TaskCreatedEvent { task_id };

        let res = Response::new()
            .add_event(task_queue_event)
            .add_submessages(hooks);

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

        if TASK_DEPOSITS.has(deps.storage, task_id) {
            TASK_DEPOSITS.remove(deps.storage, task_id);
        }

        // Prepare hooks
        let hooks = TASK_HOOKS.completed.prepare_hooks(deps.storage, |addr| {
            Ok(SubMsg::reply_on_error(
                WasmMsg::Execute {
                    contract_addr: addr.to_string(),
                    msg: to_json_binary(&TaskHookExecuteMsg::TaskCompletedHook(TaskResponse {
                        description: task.description.clone(),
                        status: task.status.clone(),
                        id: task_id,
                        payload: task.payload.clone(),
                        result: task.result.clone(),
                    }))?,
                    funds: vec![],
                },
                HOOK_ERROR_REPLY_ID,
            ))
        })?;

        let task_queue_event = TaskCompletedEvent { task_id };

        let res = Response::new()
            .add_event(task_queue_event)
            .add_submessages(hooks);

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

        // Prepare hooks
        let hooks = TASK_HOOKS.timeout.prepare_hooks(deps.storage, |addr| {
            Ok(SubMsg::reply_on_error(
                WasmMsg::Execute {
                    contract_addr: addr.to_string(),
                    msg: to_json_binary(&TaskHookExecuteMsg::TaskTimeoutHook(TaskResponse {
                        description: task.description.clone(),
                        status: task.status.clone(),
                        id: task_id,
                        payload: task.payload.clone(),
                        result: None,
                    }))?,
                    funds: vec![],
                },
                HOOK_ERROR_REPLY_ID,
            ))
        })?;

        let task_queue_event = TaskExpiredEvent { task_id };

        let mut res = Response::new()
            .add_event(task_queue_event)
            .add_submessages(hooks);

        if let Some(task_deposit) = TASK_DEPOSITS.may_load(deps.storage, task_id)? {
            res = res.add_message(BankMsg::Send {
                to_address: task_deposit.addr.to_string(),
                amount: vec![task_deposit.coin],
            });

            TASK_DEPOSITS.remove(deps.storage, task_id);
        }

        Ok(res)
    }

    pub fn add_hook(
        deps: DepsMut,
        info: MessageInfo,
        hook_type: TaskHookType,
        receiver: String,
    ) -> Result<Response, ContractError> {
        // Validate the address
        let receiver = deps.api.addr_validate(&receiver)?;

        // Only the owner can register hooks
        assert_owner(deps.storage, &info.sender)?;

        // Register the hook
        TASK_HOOKS.add_hook(deps.storage, &hook_type, receiver.clone())?;

        // Create event
        let hook_added_event = HookAddedEvent {
            hook_type,
            address: receiver.to_string(),
        };

        Ok(Response::new().add_event(hook_added_event))
    }

    pub fn remove_hook(
        deps: DepsMut,
        info: MessageInfo,
        hook_type: TaskHookType,
        receiver: String,
    ) -> Result<Response, ContractError> {
        // Validate the address
        let receiver = deps.api.addr_validate(&receiver)?;

        // Only the owner can remove hooks
        assert_owner(deps.storage, &info.sender)?;

        // Remove the hook
        TASK_HOOKS.remove_hook(deps.storage, &hook_type, receiver.clone())?;

        // Create event
        let hook_removed_event = HookRemovedEvent {
            hook_type,
            address: receiver.to_string(),
        };

        Ok(Response::new().add_event(hook_removed_event))
    }
}

mod query {
    use cosmwasm_std::Timestamp;
    use cw_ownable::get_ownership;
    use cw_storage_plus::Bound;
    use lavs_apis::{id::TaskId, tasks::ConfigResponse};

    use crate::msg::{
        CompletedTaskOverview, InfoStatus, ListCompletedResponse, ListOpenResponse, ListResponse,
        OpenTaskOverview, TaskInfoResponse, TaskResponse, TaskStatusResponse,
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
            ownership: get_ownership(deps.storage)?,
            requestor: config.requestor.into(),
            timeout: config.timeout,
            verifier: config.verifier.into_string(),
        };
        Ok(r)
    }

    // TODO: There should probably be a max page limit, but it's left unbound to keep the API simple for now
    pub fn list(
        deps: Deps,
        env: Env,
        start_after: Option<TaskId>,
        limit: Option<u32>,
    ) -> Result<ListResponse, ContractError> {
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);

        let tasks = TASKS
            .range(
                deps.storage,
                None,
                start_after.map(Bound::exclusive),
                cosmwasm_std::Order::Descending,
            )
            .map(|r| {
                r.map(|(id, task)| {
                    // add timestamps to status enum
                    let status = match task.validate_status(&env) {
                        Status::Open {} => InfoStatus::Open {
                            expires: task.timing.expires_at,
                        },
                        Status::Completed { completed, .. } => InfoStatus::Completed { completed },
                        Status::Expired {} => InfoStatus::Expired {
                            expired: task.timing.expires_at,
                        },
                    };

                    TaskInfoResponse {
                        id,
                        description: task.description,
                        status,
                        payload: task.payload,
                        result: task.result,
                        created_at: task.timing.created_at,
                    }
                })
            })
            .take(limit)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ListResponse { tasks })
    }

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
                )) if timing.expires_at > env.block.time => Some(Ok(OpenTaskOverview {
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
            .prefix(
                Status::Completed {
                    completed: Timestamp::from_seconds(0),
                }
                .as_str(),
            )
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        // Allow hooks to fail
        HOOK_ERROR_REPLY_ID => Ok(Response::default()),
        _ => Err(ContractError::UnknownReplyId { id: msg.id }),
    }
}

#[cfg(test)]
mod tests {}
