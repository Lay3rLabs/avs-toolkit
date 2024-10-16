use crate::{define_task_queue_event, id::TaskId};
use cosmwasm_std::{Attribute, Event, StdError};

use super::traits::TypedEvent;

define_task_queue_event!(TaskCreatedEvent, "task_created_event");
define_task_queue_event!(TaskCompletedEvent, "task_completed_event");
define_task_queue_event!(TaskExpiredEvent, "task_expired_event");
