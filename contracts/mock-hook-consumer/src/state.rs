use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const CREATED_COUNT: Item<u64> = Item::new("created_count");
// Stores the task queue address
// This is set when we receive a completed task hook, and we can register a hook on the newly-created task if we have whitelist permission.
pub const TASK_QUEUE: Item<Addr> = Item::new("task_queue");
