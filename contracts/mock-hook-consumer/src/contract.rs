use cosmwasm_std::{entry_point, to_json_binary, Binary, Deps, Empty, StdResult};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use execute::{add_hooks, task_completed, task_created, task_timeout};
use lavs_apis::interfaces::task_hooks::TaskHookExecuteMsg;

use crate::msg::{ExecuteMsg, QueryMsg};
use crate::state::CREATED_COUNT;

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CREATED_COUNT.save(deps.storage, &0u64)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::TaskHook(task_hook) => match task_hook {
            TaskHookExecuteMsg::TaskCreatedHook(task) => task_created(deps, env, info, task),
            TaskHookExecuteMsg::TaskCompletedHook(task) => task_completed(deps, env, info, task),
            TaskHookExecuteMsg::TaskTimeoutHook(task) => task_timeout(deps, env, info, task),
        },
        ExecuteMsg::AddHooks { task_queue } => add_hooks(deps, env, info, task_queue),
    }
}

mod execute {
    use cosmwasm_std::{to_json_binary, CosmosMsg, StdError, WasmMsg};
    use lavs_apis::{
        interfaces::task_hooks::TaskHookType,
        tasks::{ConfigResponse, Requestor, TaskResponse},
    };

    use crate::{
        msg::{TaskRequestData, TaskResponseData},
        state::CREATED_COUNT,
    };

    use super::*;

    /// Register all of the hooks on the task queue
    pub fn add_hooks(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        task_queue: String,
    ) -> StdResult<Response> {
        let task_queue = deps.api.addr_validate(&task_queue)?;

        let msgs = vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: task_queue.to_string(),
                msg: to_json_binary(&lavs_apis::tasks::ExecuteMsg::Custom(
                    lavs_apis::tasks::CustomExecuteMsg::AddHook(TaskHookType::Created),
                ))?,
                funds: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: task_queue.to_string(),
                msg: to_json_binary(&lavs_apis::tasks::ExecuteMsg::Custom(
                    lavs_apis::tasks::CustomExecuteMsg::AddHook(TaskHookType::Completed),
                ))?,
                funds: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: task_queue.to_string(),
                msg: to_json_binary(&lavs_apis::tasks::ExecuteMsg::Custom(
                    lavs_apis::tasks::CustomExecuteMsg::AddHook(TaskHookType::Timeout),
                ))?,
                funds: vec![],
            }),
        ];

        Ok(Response::default().add_messages(msgs))
    }

    /// For a task created, we want to increase our created counter.
    pub fn task_created(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _task: TaskResponse,
    ) -> StdResult<Response> {
        CREATED_COUNT.update(deps.storage, |x| -> StdResult<_> { Ok(x + 1) })?;

        Ok(Response::default())
    }

    /// For the task completed, we want to create another task on the task queue.
    pub fn task_completed(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        task: TaskResponse,
    ) -> StdResult<Response> {
        let task_queue = info.sender;

        // Attempt to deserialize the task response
        let response: TaskResponseData =
            serde_json::from_value(task.result.expect("Result is not available")).map_err(|e| {
                StdError::generic_err(format!(
                    "Could not deserialize input request from JSON: {}",
                    e
                ))
            })?;

        // Query requestor config
        let config: ConfigResponse = deps.querier.query_wasm_smart(
            task_queue.to_string(),
            &lavs_apis::tasks::QueryMsg::Custom(lavs_apis::tasks::CustomQueryMsg::Config {}),
        )?;

        // Get amount to send
        let funds = match config.requestor {
            Requestor::Fixed(addr) => {
                if addr == env.contract.address.to_string() {
                    Ok(vec![])
                } else {
                    Err(StdError::generic_err(
                        "Contract is not authorized to create tasks",
                    ))
                }
            }
            Requestor::OpenPayment(coin) => Ok(vec![coin]),
        }?;

        // Construct a new request to square the square result
        let request = TaskRequestData { x: response.y };

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: task_queue.to_string(),
            msg: to_json_binary(&lavs_apis::tasks::ExecuteMsg::Custom(
                lavs_apis::tasks::CustomExecuteMsg::Create {
                    description: task.description,
                    timeout: None,
                    payload: serde_json::to_value(request).map_err(|e| {
                        StdError::generic_err(format!(
                            "Could not serialize request into JSON: {}",
                            e
                        ))
                    })?,
                },
            ))?,
            funds,
        });

        Ok(Response::default()
            .add_attribute("action", "task_completed")
            .add_message(msg))
    }

    /// For the task timeout, we will throw an error to showcase the non-blocking workflow.
    pub fn task_timeout(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _task: TaskResponse,
    ) -> StdResult<Response> {
        Err(StdError::generic_err("This is an error"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CreatedCount {} => {
            to_json_binary(&CREATED_COUNT.may_load(deps.storage)?.unwrap_or_default())
        }
    }
}
