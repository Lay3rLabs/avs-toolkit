use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Deps, Env, MessageInfo, StdError, Timestamp};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use cw_utils::must_pay;

use lavs_apis::id::TaskId;
use lavs_apis::interfaces::task_hooks::TaskHooks;
use lavs_apis::tasks::{Requestor, Status, TimeoutConfig};
use lavs_apis::time::Duration;

use crate::error::ContractError;
use crate::msg::{self, InstantiateMsg, RequestType, ResponseType};

pub const CONFIG: Item<Config> = Item::new("config");
pub const TASK_HOOKS: TaskHooks = TaskHooks::new(
    "global_hooks",
    "task_specific_hooks",
    "task_specific_whitelist",
);
pub const TASK_DEPOSITS: Map<TaskId, TaskDeposit> = Map::new("task_deposits");

pub struct TaskIndexes<'a> {
    pub status: MultiIndex<'a, &'a str, Task, TaskId>,
}

impl<'a> IndexList<Task> for TaskIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Task>> + '_> {
        Box::new(std::iter::once(&self.status as &dyn Index<Task>))
    }
}

pub const TASKS: IndexedMap<TaskId, Task, TaskIndexes<'static>> = IndexedMap::new(
    "tasks",
    TaskIndexes {
        status: MultiIndex::new(|_, d: &Task| d.status.as_str(), "tasks", "tasks__status"),
    },
);

#[cw_serde]
pub struct Config {
    pub next_id: TaskId,
    pub requestor: RequestorConfig,
    pub timeout: TimeoutConfig,
    pub verifier: Addr,
}

#[cw_serde]
pub struct TaskDeposit {
    pub addr: Addr,
    /// We store a coin here in case we ever want to support updates to the RequestorConfig
    pub coin: Coin,
}

impl Config {
    pub fn validate(deps: Deps, input: InstantiateMsg) -> Result<Self, ContractError> {
        let requestor = RequestorConfig::validate(deps, input.requestor)?;
        let timeout = validate_timeout_info(input.timeout)?;
        let verifier = deps.api.addr_validate(&input.verifier)?;
        Ok(Config {
            next_id: TaskId::new(1),
            requestor,
            timeout,
            verifier,
        })
    }
}

#[cw_serde]
pub enum RequestorConfig {
    Fixed(Addr),
    OpenPayment(Coin),
}

impl RequestorConfig {
    pub fn validate(deps: Deps, input: msg::Requestor) -> Result<Self, StdError> {
        match input {
            msg::Requestor::Fixed(addr) => {
                Ok(RequestorConfig::Fixed(deps.api.addr_validate(&addr)?))
            }
            msg::Requestor::OpenPayment(coin) => Ok(RequestorConfig::OpenPayment(coin)),
        }
    }

    pub fn check_requestor(&self, info: &MessageInfo) -> Result<(), ContractError> {
        match self {
            RequestorConfig::Fixed(addr) => {
                if info.sender != addr {
                    return Err(ContractError::Unauthorized);
                }
            }
            RequestorConfig::OpenPayment(needed) => {
                let paid = must_pay(info, &needed.denom)?;
                if paid < needed.amount {
                    return Err(ContractError::InsufficientPayment(
                        needed.amount.u128(),
                        needed.denom.clone(),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl From<RequestorConfig> for Requestor {
    fn from(val: RequestorConfig) -> Self {
        match val {
            RequestorConfig::Fixed(addr) => Requestor::Fixed(addr.into_string()),
            RequestorConfig::OpenPayment(coin) => Requestor::OpenPayment(coin),
        }
    }
}

pub fn validate_timeout_info(input: msg::TimeoutInfo) -> Result<TimeoutConfig, ContractError> {
    let default = input.default;
    let minimum = input.minimum.unwrap_or(default);
    let maximum = input.maximum.unwrap_or(default);
    if default < minimum || default > maximum || minimum > maximum {
        return Err(ContractError::InvalidTimeoutInfo);
    }
    Ok(TimeoutConfig {
        default,
        minimum,
        maximum,
    })
}

pub fn check_timeout(
    config: &TimeoutConfig,
    timeout: Option<Duration>,
) -> Result<Duration, ContractError> {
    match timeout {
        Some(t) if t < config.minimum => Err(ContractError::TimeoutTooShort(config.minimum)),
        Some(t) if t > config.maximum => Err(ContractError::TimeoutTooLong(config.maximum)),
        Some(t) => Ok(t),
        None => Ok(config.default),
    }
}

#[cw_serde]
pub struct Task {
    pub creator: Addr,
    pub description: String,
    pub status: Status,
    pub timing: Timing,
    pub payload: RequestType,
    pub result: Option<ResponseType>,
}

impl Task {
    pub fn validate_status(&self, env: &Env) -> Status {
        match self.status {
            Status::Open {} if !self.timing.is_expired(env) => self.status.clone(),
            Status::Expired {} | Status::Open {} => Status::Expired {},
            Status::Completed { .. } => self.status.clone(),
        }
    }
}

#[cw_serde]
pub struct Timing {
    /// Creation in `Timestamp` format
    pub created_at: Timestamp,
    /// Expiration in `Timestamp` format
    pub expires_at: Timestamp,
    /// Creation in block height
    pub created_height: u64,
}

impl Timing {
    pub fn new(env: &Env, timeout_duration: Duration) -> Self {
        Timing {
            created_at: env.block.time,
            expires_at: env.block.time.plus_nanos(timeout_duration.as_nanos()),
            created_height: env.block.height,
        }
    }

    pub fn is_expired(&self, env: &Env) -> bool {
        self.expires_at <= env.block.time
    }
}

impl Task {
    pub fn complete(&mut self, env: &Env, result: ResponseType) -> Result<(), ContractError> {
        match self.status {
            Status::Open {} if !self.timing.is_expired(env) => {}
            Status::Open {} | Status::Expired {} => return Err(ContractError::TaskExpired),
            Status::Completed { .. } => return Err(ContractError::TaskCompleted),
        };
        self.status = Status::completed(env);
        self.result = Some(result);
        Ok(())
    }

    pub fn expire(&mut self, env: &Env) -> Result<(), ContractError> {
        match self.status {
            Status::Open {} if self.timing.is_expired(env) => {}
            Status::Open {} => return Err(ContractError::TaskNotExpired),
            Status::Expired {} => return Err(ContractError::TaskExpired),
            Status::Completed { .. } => return Err(ContractError::TaskCompleted),
        };
        self.status = Status::Expired {};
        Ok(())
    }
}
