use cosmwasm_std::{entry_point, to_json_binary, Binary, Deps, StdError, StdResult};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use execute::*;
use lavs_apis::interfaces::task_hooks::TaskHookExecuteMsg;

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{CW4_GROUP, DAO, DECISIONS, NEW_MEMBER_WEIGHT, TASK_QUEUE};

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    DAO.save(deps.storage, &deps.api.addr_validate(&msg.dao)?)?;
    CW4_GROUP.save(deps.storage, &deps.api.addr_validate(&msg.cw4_group)?)?;
    TASK_QUEUE.save(deps.storage, &deps.api.addr_validate(&msg.task_queue)?)?;
    NEW_MEMBER_WEIGHT.save(deps.storage, &msg.new_member_weight)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Trigger {
            message_id,
            message,
        } => trigger(deps, env, info, message_id, message),
        ExecuteMsg::Unregister {} => unregister(deps),
        ExecuteMsg::UpdateDao { dao } => update_dao(deps, info, dao),
        ExecuteMsg::UpdateCw4Group { cw4_group } => update_cw4_group(deps, info, cw4_group),
        ExecuteMsg::UpdateTaskQueue { task_queue } => update_task_queue(deps, info, task_queue),
        ExecuteMsg::UpdateNewMemberWeight { weight } => {
            update_new_member_weight(deps, info, weight)
        }
        ExecuteMsg::Cw4Group(msg) => execute_cw4_group(deps, info, msg),
        ExecuteMsg::TaskHook(task_hook) => match task_hook {
            TaskHookExecuteMsg::TaskCompletedHook(task) => Ok(task_completed_hook(
                deps, env, info, task,
            )
            .unwrap_or_else(|e| {
                Response::default()
                    .add_attribute("action", "task_completed")
                    .add_attribute("error", e.to_string())
            })),
            _ => Err(StdError::generic_err("unexpected task hook")),
        },
    }
}

mod execute {
    use cosmwasm_std::{to_json_binary, CosmosMsg, StdError, WasmMsg};
    use lavs_apis::tasks::TaskResponse;

    use crate::msg::{TaskInput, TaskOutput};

    use super::*;

    pub fn trigger(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        message_id: u16,
        message: String,
    ) -> StdResult<Response> {
        let task_queue = TASK_QUEUE.load(deps.storage)?;
        let dao = DAO.load(deps.storage)?;
        let address = info.sender;

        let payload = serde_json::to_value(TaskInput {
            dao: dao.to_string(),
            address: address.to_string(),
            message_id,
            message,
        })
        .map_err(|e| StdError::generic_err(e.to_string()))?;

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: task_queue.to_string(),
            msg: to_json_binary(&lavs_apis::tasks::ExecuteMsg::Custom(
                lavs_apis::tasks::CustomExecuteMsg::Create {
                    description: "AI Bouncer Message".to_string(),
                    timeout: None,
                    payload,
                    with_timeout_hooks: None,
                    // atomic completion hook to this contract
                    with_completed_hooks: Some(vec![env.contract.address.to_string()]),
                },
            ))?,
            funds: info.funds,
        });

        Ok(Response::default()
            .add_attribute("action", "trigger")
            .add_attribute("dao", dao.to_string())
            .add_attribute("address", address.to_string())
            .add_attribute("message_id", message_id.to_string())
            .add_message(msg))
    }

    pub fn unregister(deps: DepsMut) -> StdResult<Response> {
        let dao = DAO.load(deps.storage)?;
        let cw4_group = CW4_GROUP.load(deps.storage)?;

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cw4_group.to_string(),
            msg: to_json_binary(&cw4_group::msg::ExecuteMsg::UpdateAdmin {
                admin: Some(dao.to_string()),
            })?,
            funds: vec![],
        });

        Ok(Response::default()
            .add_attribute("action", "unregister")
            .add_message(msg))
    }

    pub fn update_dao(deps: DepsMut, info: MessageInfo, dao: String) -> StdResult<Response> {
        if info.sender != DAO.load(deps.storage)? {
            return Err(StdError::generic_err("unauthorized: not DAO"));
        }

        DAO.save(deps.storage, &deps.api.addr_validate(&dao)?)?;
        Ok(Response::default()
            .add_attribute("action", "update_dao")
            .add_attribute("dao", dao))
    }

    pub fn update_cw4_group(
        deps: DepsMut,
        info: MessageInfo,
        cw4_group: String,
    ) -> StdResult<Response> {
        if info.sender != DAO.load(deps.storage)? {
            return Err(StdError::generic_err("unauthorized: not DAO"));
        }

        CW4_GROUP.save(deps.storage, &deps.api.addr_validate(&cw4_group)?)?;
        Ok(Response::default()
            .add_attribute("action", "update_cw4_group")
            .add_attribute("cw4_group", cw4_group))
    }

    pub fn update_task_queue(
        deps: DepsMut,
        info: MessageInfo,
        task_queue: String,
    ) -> StdResult<Response> {
        if info.sender != DAO.load(deps.storage)? {
            return Err(StdError::generic_err("unauthorized: not DAO"));
        }

        TASK_QUEUE.save(deps.storage, &deps.api.addr_validate(&task_queue)?)?;
        Ok(Response::default()
            .add_attribute("action", "update_task_queue")
            .add_attribute("task_queue", task_queue))
    }

    pub fn update_new_member_weight(
        deps: DepsMut,
        info: MessageInfo,
        weight: u64,
    ) -> StdResult<Response> {
        if info.sender != DAO.load(deps.storage)? {
            return Err(StdError::generic_err("unauthorized: not DAO"));
        }

        NEW_MEMBER_WEIGHT.save(deps.storage, &weight)?;
        Ok(Response::default()
            .add_attribute("action", "update_new_member_weight")
            .add_attribute("weight", weight.to_string()))
    }

    pub fn execute_cw4_group(
        deps: DepsMut,
        info: MessageInfo,
        msg: cw4_group::msg::ExecuteMsg,
    ) -> StdResult<Response> {
        if info.sender != DAO.load(deps.storage)? {
            return Err(StdError::generic_err("unauthorized: not DAO"));
        }

        let cw4_group = CW4_GROUP.load(deps.storage)?;
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cw4_group.to_string(),
            msg: to_json_binary(&msg)?,
            funds: info.funds,
        });

        Ok(Response::default()
            .add_attribute("action", "execute_cw4_group")
            .add_attribute("cw4_group", cw4_group)
            .add_message(msg))
    }

    pub fn task_completed_hook(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        task: TaskResponse,
    ) -> StdResult<Response> {
        if info.sender != TASK_QUEUE.load(deps.storage)? {
            return Err(StdError::generic_err("unexpected task queue"));
        }

        // Attempt to deserialize the task response
        let response: TaskOutput =
            serde_json::from_value(task.result.expect("Result is not available")).map_err(|e| {
                StdError::generic_err(format!(
                    "Could not deserialize input request from JSON: {}",
                    e
                ))
            })?;

        if let TaskOutput::Success(success) = response {
            if let Some(decision) = success.decision {
                let address = deps.api.addr_validate(&success.address)?;

                if DECISIONS.has(deps.storage, &address) {
                    return Ok(Response::default()
                        .add_attribute("action", "task_completed")
                        .add_attribute("decision", "already_decided"));
                }

                DECISIONS.save(deps.storage, &address, &decision)?;

                let mut response = Response::default()
                    .add_attribute("action", "task_completed")
                    .add_attribute("decision", decision.to_string());

                if decision {
                    let cw4_group = CW4_GROUP.load(deps.storage)?;
                    let weight = NEW_MEMBER_WEIGHT.load(deps.storage)?;

                    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: cw4_group.to_string(),
                        msg: to_json_binary(&cw4_group::msg::ExecuteMsg::UpdateMembers {
                            add: vec![cw4::Member {
                                addr: address.to_string(),
                                weight,
                            }],
                            remove: vec![],
                        })?,
                        funds: vec![],
                    });
                    response = response.add_message(msg);
                }

                return Ok(response);
            }
        }

        Ok(Response::default()
            .add_attribute("action", "task_completed")
            .add_attribute("decision", "none"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Decision { address } => {
            let address = deps.api.addr_validate(&address)?;
            let decision = DECISIONS.may_load(deps.storage, &address)?;
            to_json_binary(&decision)
        }
        QueryMsg::Dao {} => to_json_binary(&DAO.load(deps.storage)?),
        QueryMsg::Cw4Group {} => to_json_binary(&CW4_GROUP.load(deps.storage)?),
        QueryMsg::TaskQueue {} => to_json_binary(&TASK_QUEUE.load(deps.storage)?),
        QueryMsg::NewMemberWeight {} => to_json_binary(&NEW_MEMBER_WEIGHT.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
