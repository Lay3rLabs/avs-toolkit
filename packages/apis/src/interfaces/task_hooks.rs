use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_json_binary, Addr, Api, CustomQuery, Deps, StdError, StdResult, Storage, SubMsg,
};
use cw_storage_plus::Map;
use thiserror::Error;

use crate::{id::TaskId, tasks::TaskResponse};

#[cw_serde]
/// Maintains cw-controllers interface here
pub struct HooksResponse {
    pub hooks: Vec<String>,
}

pub struct TaskHooks<'a> {
    /// Hooks executed on every task of the task queue
    /// hook_type -> receiver
    pub global_hooks: Map<&'a str, Vec<Addr>>,
    /// Hooks executed only on specific tasks of the task queue
    /// (task_id, hook_type) -> receivers
    pub task_specific_hooks: Map<(TaskId, &'a str), Vec<Addr>>,
    /// Whitelist of addresses allowed to submit a hook for their submissions
    pub task_specific_whitelist: Map<&'a Addr, ()>,
}

#[cw_serde]
pub struct TaskHookPayload {
    pub task_id: TaskId,
    pub hook_type: TaskHookType,
    pub addr: Addr,
}

impl<'a> TaskHooks<'a> {
    pub const fn new(
        global_hooks: &'static str,
        task_specific_hooks: &'static str,
        task_specific_whitelist: &'static str,
    ) -> Self {
        Self {
            global_hooks: Map::new(global_hooks),
            task_specific_hooks: Map::new(task_specific_hooks),
            task_specific_whitelist: Map::new(task_specific_whitelist),
        }
    }

    /// Updates the whitelist of addresses allowed to submit a hook for their submission
    pub fn update_task_specific_whitelist(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    ) -> Result<(), TaskHookError> {
        // Add new addresses to whitelist
        if let Some(addresses) = to_add {
            for addr in addresses {
                let addr = api.addr_validate(&addr)?;
                self.task_specific_whitelist.save(storage, &addr, &())?;
            }
        }

        // Remove addresses from whitelist
        if let Some(addresses) = to_remove {
            for addr in addresses {
                let addr = api.addr_validate(&addr)?;
                self.task_specific_whitelist.remove(storage, &addr);
            }
        }

        Ok(())
    }

    /// Adds a hook for either a specific task or globally
    /// Returns HookAlreadyRegistered error if the hook already exists
    pub fn add_hook(
        &self,
        storage: &mut dyn Storage,
        is_task_created: bool,
        task_id: Option<TaskId>,
        hook_type: &TaskHookType,
        addr: Addr,
    ) -> Result<(), TaskHookError> {
        let hook_type_str = hook_type.as_str();

        match task_id {
            Some(id) => {
                // Do not allow creation of a task hook on an already created task
                // We should allow hooks for future tasks though
                if matches!(hook_type, TaskHookType::Created) && is_task_created {
                    return Err(TaskHookError::TaskAlreadyCreated {});
                }

                let key = (id, hook_type_str);
                let mut hooks = self
                    .task_specific_hooks
                    .may_load(storage, key)?
                    .unwrap_or_default();

                if !hooks.contains(&addr) {
                    hooks.push(addr);
                    self.task_specific_hooks.save(storage, key, &hooks)?;
                } else {
                    return Err(TaskHookError::HookAlreadyRegistered {});
                }
            }
            None => {
                let mut hooks = self
                    .global_hooks
                    .may_load(storage, hook_type_str)?
                    .unwrap_or_default();

                if !hooks.contains(&addr) {
                    hooks.push(addr);
                    self.global_hooks.save(storage, hook_type_str, &hooks)?;
                } else {
                    return Err(TaskHookError::HookAlreadyRegistered {});
                }
            }
        }
        Ok(())
    }

    pub fn remove_hook(
        &self,
        storage: &mut dyn Storage,
        task_id: Option<TaskId>,
        hook_type: &TaskHookType,
        addr: Addr,
    ) -> Result<(), TaskHookError> {
        let hook_type_str = hook_type.as_str();

        match task_id {
            Some(id) => {
                let key = (id, hook_type_str);
                let mut hooks = self.task_specific_hooks.load(storage, key)?;

                if let Some(pos) = hooks.iter().position(|x| x == addr) {
                    hooks.remove(pos);

                    if hooks.is_empty() {
                        self.task_specific_hooks.remove(storage, key);
                    } else {
                        self.task_specific_hooks.save(storage, key, &hooks)?;
                    }
                } else {
                    return Err(TaskHookError::HookNotRegistered {});
                }
            }
            None => {
                let mut hooks = self.global_hooks.load(storage, hook_type_str)?;

                if let Some(pos) = hooks.iter().position(|x| x == addr) {
                    hooks.remove(pos);

                    if hooks.is_empty() {
                        self.global_hooks.remove(storage, hook_type_str);
                    } else {
                        self.global_hooks.save(storage, hook_type_str, &hooks)?;
                    };
                } else {
                    return Err(TaskHookError::HookNotRegistered {});
                }
            }
        }
        Ok(())
    }

    /// Prepares hook messages for both global and task-specific hooks
    /// Always includes global hooks and adds task-specific hooks if they exist
    pub fn prepare_hooks<F: Fn(Addr) -> StdResult<SubMsg>>(
        &self,
        storage: &dyn Storage,
        task_id: TaskId,
        hook_type: TaskHookType,
        prep: F,
    ) -> StdResult<Vec<SubMsg>> {
        let hook_type_str = hook_type.as_str();
        let mut msgs = Vec::new();

        // Always include global hooks
        if let Some(hooks) = self.global_hooks.may_load(storage, hook_type_str)? {
            for hook in hooks {
                msgs.push(prep(hook)?);
            }
        }

        // Add task-specific hooks
        if let Some(hooks) = self
            .task_specific_hooks
            .may_load(storage, (task_id, hook_type_str))?
        {
            for hook in hooks {
                // Task-specific hooks should have a payload for unregistering the hook automatically
                msgs.push(
                    prep(hook.clone())?.with_payload(to_json_binary(&TaskHookPayload {
                        task_id,
                        hook_type: hook_type.clone(),
                        addr: hook,
                    })?),
                );
            }
        }

        Ok(msgs)
    }

    pub fn query_hooks<Q: CustomQuery>(
        &self,
        deps: Deps<Q>,
        task_id: Option<TaskId>,
        hook_type: TaskHookType,
    ) -> StdResult<HooksResponse> {
        let hook_type_str = hook_type.as_str();
        let mut hooks = Vec::new();

        // Get global hooks
        if let Some(global_hooks) = self.global_hooks.may_load(deps.storage, hook_type_str)? {
            hooks.extend(global_hooks.into_iter().map(String::from));
        }

        // Get task-specific hooks if task_id is provided
        if let Some(id) = task_id {
            if let Some(task_hooks) = self
                .task_specific_hooks
                .may_load(deps.storage, (id, hook_type_str))?
            {
                hooks.extend(task_hooks.into_iter().map(String::from));
            }
        }

        Ok(HooksResponse { hooks })
    }
}

#[cw_serde]
pub enum TaskHookType {
    Created,
    Completed,
    Timeout,
}

impl TaskHookType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            TaskHookType::Created => "created",
            TaskHookType::Completed => "completed",
            TaskHookType::Timeout => "timeout",
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum TaskHookError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Given address already registered as a hook")]
    HookAlreadyRegistered {},

    #[error("Given address not registered as a hook")]
    HookNotRegistered {},

    #[error("Cannot add a created hook to an existing task")]
    TaskAlreadyCreated {},
}

#[cw_serde]
pub enum TaskHookExecuteMsg {
    TaskCompletedHook(TaskResponse),
    TaskTimeoutHook(TaskResponse),
    TaskCreatedHook(TaskResponse),
}
