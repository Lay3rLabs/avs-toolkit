use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// The DAO.
pub const DAO: Item<Addr> = Item::new("dao");

/// The cw4-group.
pub const CW4_GROUP: Item<Addr> = Item::new("group");

/// The task queue.
pub const TASK_QUEUE: Item<Addr> = Item::new("task_queue");

/// The new member weight.
pub const NEW_MEMBER_WEIGHT: Item<u64> = Item::new("new_member_weight");

/// Whether or not an address was approved and added to the DAO, or rejected,
/// preventing further attempts to add them.
pub const DECISIONS: Map<&Addr, bool> = Map::new("decisions");
