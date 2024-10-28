use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, StdResult, Storage};
use cw_controllers::{HookError, Hooks, HooksResponse};

use crate::tasks::TaskResponse;

pub struct TaskHooks {
    pub completed: Hooks,
    pub timeout: Hooks,
    pub created: Hooks,
}

impl TaskHooks {
    pub const fn new(
        completed: &'static str,
        timeout: &'static str,
        created: &'static str,
    ) -> Self {
        Self {
            completed: Hooks::new(completed),
            timeout: Hooks::new(timeout),
            created: Hooks::new(created),
        }
    }

    pub fn add_hook(
        &self,
        storage: &mut dyn Storage,
        task_hook_type: &TaskHookType,
        addr: Addr,
    ) -> Result<(), HookError> {
        match task_hook_type {
            TaskHookType::Completed => self.completed.add_hook(storage, addr),
            TaskHookType::Timeout => self.timeout.add_hook(storage, addr),
            TaskHookType::Created => self.created.add_hook(storage, addr),
        }?;

        Ok(())
    }

    pub fn remove_hook(
        &self,
        storage: &mut dyn Storage,
        task_hook_type: &TaskHookType,
        addr: Addr,
    ) -> Result<(), HookError> {
        match task_hook_type {
            TaskHookType::Completed => self.completed.remove_hook(storage, addr),
            TaskHookType::Timeout => self.timeout.remove_hook(storage, addr),
            TaskHookType::Created => self.created.remove_hook(storage, addr),
        }?;

        Ok(())
    }

    pub fn query_hooks(
        &self,
        deps: Deps,
        task_hook_type: TaskHookType,
    ) -> StdResult<HooksResponse> {
        match task_hook_type {
            TaskHookType::Completed => self.completed.query_hooks(deps),
            TaskHookType::Timeout => self.timeout.query_hooks(deps),
            TaskHookType::Created => self.created.query_hooks(deps),
        }
    }
}

impl Default for TaskHooks {
    fn default() -> Self {
        Self::new(
            "task_completed_hooks",
            "task_timeout_hooks",
            "task_created_hooks",
        )
    }
}

#[cw_serde]
pub enum TaskHookType {
    Completed,
    Timeout,
    Created,
}

#[cw_serde]
pub enum TaskHookExecuteMsg {
    TaskCompletedHook(TaskResponse),
    TaskTimeoutHook(TaskResponse),
    TaskCreatedHook(TaskResponse),
}
