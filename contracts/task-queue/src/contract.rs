#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_json, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, Reply,
    Response, WasmMsg,
};
use cw2::set_contract_version;

use lavs_apis::interfaces::task_hooks::TaskHookPayload;
use lavs_apis::interfaces::tasks as interface;
use lavs_apis::tasks::{CustomExecuteMsg, CustomQueryMsg, TaskQueryMsg};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::msg::{RequestType, ResponseType, Status};
use crate::state::{Config, Task, CONFIG, TASKS, TASK_HOOKS};

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const TASK_HOOK_REPLY_ID: u64 = 0;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let owner = msg.owner.clone().unwrap_or(info.sender.to_string());

    let msgs = if msg.task_specific_whitelist.is_some() {
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_json_binary(&ExecuteMsg::Custom(
                CustomExecuteMsg::UpdateTaskSpecificWhitelist {
                    to_add: msg.task_specific_whitelist.clone(),
                    to_remove: None,
                },
            ))?,
            funds: vec![],
        })]
    } else {
        vec![]
    };

    let config = Config::validate(deps.as_ref(), msg)?;
    CONFIG.save(deps.storage, &config)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&owner))?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default().add_messages(msgs))
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
                with_completed_hooks,
                with_timeout_hooks,
            } => execute::create(
                deps,
                env,
                info,
                description,
                timeout,
                payload,
                with_completed_hooks,
                with_timeout_hooks,
            ),
            CustomExecuteMsg::Timeout { task_id } => execute::timeout(deps, env, info, task_id),
            CustomExecuteMsg::AddHooks {
                task_id,
                hook_type,
                receivers,
            } => execute::add_hooks(deps, env, info, task_id, hook_type, receivers),
            CustomExecuteMsg::RemoveHook {
                task_id,
                hook_type,
                receiver,
            } => execute::remove_hook(deps, info, task_id, hook_type, receiver),
            CustomExecuteMsg::UpdateOwnership(action) => {
                let ownership =
                    cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;

                let event =
                    Event::new("update_ownership").add_attributes(ownership.into_attributes());

                Ok(Response::new().add_event(event))
            }
            CustomExecuteMsg::UpdateTaskSpecificWhitelist { to_add, to_remove } => {
                execute::update_task_specific_whitelist(deps, env, info, to_add, to_remove)
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
            CustomQueryMsg::TaskHooks { hook_type, task_id } => Ok(to_json_binary(
                &TASK_HOOKS.query_hooks(deps, task_id, hook_type)?,
            )?),
            CustomQueryMsg::Ownership {} => {
                Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?)
            }
            CustomQueryMsg::TaskSpecificWhitelist { start_after, limit } => Ok(to_json_binary(
                &query::task_specific_whitelist(deps, start_after, limit)?,
            )?),
        },
    }
}

mod execute {
    use cosmwasm_std::{ensure, BankMsg, SubMsg, WasmMsg};
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

    #[allow(clippy::too_many_arguments)]
    pub fn create(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        description: String,
        timeout: Option<Duration>,
        payload: RequestType,
        with_completed_hooks: Option<Vec<String>>,
        with_timeout_hooks: Option<Vec<String>>,
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
            creator: info.sender.clone(),
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
                    addr: info.sender.clone(),
                    coin,
                },
            )?;
        }

        // Prepare hooks
        let hooks =
            TASK_HOOKS.prepare_hooks(deps.storage, task_id, TaskHookType::Created, |addr| {
                Ok(SubMsg::reply_always(
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
                    TASK_HOOK_REPLY_ID,
                ))
            })?;

        let mut add_hooks_msgs = vec![];

        // Validate task-specific whitelist if we have some atomic hooks
        if with_completed_hooks.is_some() || with_timeout_hooks.is_some() {
            ensure!(
                TASK_HOOKS
                    .task_specific_whitelist
                    .has(deps.storage, &info.sender)
                    || assert_owner(deps.storage, &info.sender).is_ok(),
                ContractError::Unauthorized
            )
        }
        if let Some(receivers) = with_completed_hooks {
            add_hooks_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_json_binary(&ExecuteMsg::Custom(CustomExecuteMsg::AddHooks {
                    task_id: Some(task_id),
                    hook_type: TaskHookType::Completed,
                    receivers,
                }))?,
                funds: vec![],
            }))
        }
        if let Some(receivers) = with_timeout_hooks {
            add_hooks_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_json_binary(&ExecuteMsg::Custom(CustomExecuteMsg::AddHooks {
                    task_id: Some(task_id),
                    hook_type: TaskHookType::Timeout,
                    receivers,
                }))?,
                funds: vec![],
            }))
        }

        let task_queue_event = TaskCreatedEvent { task_id };

        let res = Response::new()
            .add_event(task_queue_event)
            .add_messages(add_hooks_msgs)
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
        let hooks =
            TASK_HOOKS.prepare_hooks(deps.storage, task_id, TaskHookType::Completed, |addr| {
                Ok(SubMsg::reply_always(
                    WasmMsg::Execute {
                        contract_addr: addr.to_string(),
                        msg: to_json_binary(&TaskHookExecuteMsg::TaskCompletedHook(
                            TaskResponse {
                                description: task.description.clone(),
                                status: task.status.clone(),
                                id: task_id,
                                payload: task.payload.clone(),
                                result: task.result.clone(),
                            },
                        ))?,
                        funds: vec![],
                    },
                    TASK_HOOK_REPLY_ID,
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
        let hooks =
            TASK_HOOKS.prepare_hooks(deps.storage, task_id, TaskHookType::Timeout, |addr| {
                Ok(SubMsg::reply_always(
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
                    TASK_HOOK_REPLY_ID,
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

    pub fn add_hooks(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        task_id: Option<TaskId>,
        hook_type: TaskHookType,
        receivers: Vec<String>,
    ) -> Result<Response, ContractError> {
        // This method assumes an authorization check was done at task creation
        if info.sender != env.contract.address {
            // Only the owner - or task-specific whitelisted accounts for their own tasks - can register hooks
            if assert_owner(deps.storage, &info.sender).is_err() {
                if !TASK_HOOKS
                    .task_specific_whitelist
                    .has(deps.storage, &info.sender)
                {
                    // If the sender is not in the task specific whitelist, then error out
                    return Err(ContractError::Unauthorized);
                } else if let Some(task_id) = task_id {
                    let maybe_task = TASKS.may_load(deps.storage, task_id)?;

                    if let Some(task) = maybe_task {
                        // If task exists, but the creator is not the sender, then error out
                        if task.creator != info.sender {
                            return Err(ContractError::Unauthorized);
                        }
                        // Succesful auth here
                    } else {
                        // If task does not exist, then we cannot determine the creator to authorize
                        return Err(ContractError::Unauthorized);
                    }
                } else {
                    // If a task id is not provided, then we can't go through the task-specific whitelist flow
                    return Err(ContractError::Unauthorized);
                }
            }
        }

        // Register the hook
        let is_task_created = if let Some(task_id) = task_id {
            TASKS.has(deps.storage, task_id)
        } else {
            false
        };

        let mut response = Response::new();
        for receiver in receivers {
            // Validate the address
            let receiver = deps.api.addr_validate(&receiver)?;

            TASK_HOOKS.add_hook(
                deps.storage,
                is_task_created, // Param to determine if we can add a hook for the Created status
                task_id,
                &hook_type,
                receiver.clone(),
            )?;

            // Create event
            let hook_added_event = HookAddedEvent {
                hook_type: hook_type.clone(),
                address: receiver.to_string(),
            };

            response = response.add_event(hook_added_event);
        }

        Ok(response)
    }

    pub fn remove_hook(
        deps: DepsMut,
        info: MessageInfo,
        task_id: Option<TaskId>,
        hook_type: TaskHookType,
        receiver: String,
    ) -> Result<Response, ContractError> {
        // Validate the address
        let receiver = deps.api.addr_validate(&receiver)?;

        // Only the owner can remove hooks
        assert_owner(deps.storage, &info.sender)?;

        // Remove the hook
        TASK_HOOKS.remove_hook(deps.storage, task_id, &hook_type, receiver.clone())?;

        // Create event
        let hook_removed_event = HookRemovedEvent {
            hook_type,
            address: receiver.to_string(),
        };

        Ok(Response::new().add_event(hook_removed_event))
    }

    pub fn update_task_specific_whitelist(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    ) -> Result<Response, ContractError> {
        // Allow the whitelist to be set at instantiation
        if info.sender != env.contract.address {
            assert_owner(deps.storage, &info.sender)?;
        }

        TASK_HOOKS.update_task_specific_whitelist(deps.api, deps.storage, to_add, to_remove)?;

        Ok(Response::new().add_attribute("action", "Update_task_specific_whitelist"))
    }
}

mod query {
    use cosmwasm_std::Timestamp;
    use cw_ownable::get_ownership;
    use cw_storage_plus::Bound;
    use lavs_apis::{
        id::TaskId,
        tasks::{ConfigResponse, TaskSpecificWhitelistResponse},
    };

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

    pub fn task_specific_whitelist(
        deps: Deps,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<TaskSpecificWhitelistResponse, ContractError> {
        let limit = limit.unwrap_or(30);
        let binding = start_after
            .map(|x| deps.api.addr_validate(&x))
            .transpose()?;
        let start_after = binding.as_ref().map(Bound::exclusive);

        let addrs = TASK_HOOKS
            .task_specific_whitelist
            .keys(
                deps.storage,
                start_after,
                None,
                cosmwasm_std::Order::Ascending,
            )
            .take(limit as usize)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TaskSpecificWhitelistResponse { addrs })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        TASK_HOOK_REPLY_ID => {
            if let Ok(payload) = from_json::<TaskHookPayload>(msg.payload) {
                // If we have a valid TaskHookPayload, then we can remove the task-specific hook
                TASK_HOOKS.remove_hook(
                    deps.storage,
                    Some(payload.task_id),
                    &payload.hook_type,
                    payload.addr,
                )?;
            }

            // Handle any result as valid to prevent blocking
            Ok(Response::default())
        }
        _ => Err(ContractError::UnknownReplyId { id: msg.id }),
    }
}

#[cfg(test)]
mod tests {}
