use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Deps, Env, MessageInfo, StdError};
use cw_storage_plus::{Item, Map};
use cw_utils::must_pay;

use lavs_apis::tasks::{Requestor, Status};

use crate::error::ContractError;
use crate::msg::{self, InstantiateMsg, RequestType, ResponseType};

pub const CONFIG: Item<Config> = Item::new("config");
pub const TASKS: Map<u64, Task> = Map::new("tasks");

#[cw_serde]
pub struct Config {
    pub next_id: u64,
    pub requestor: RequestorConfig,
    pub timeout: TimeoutConfig,
    pub verifier: Addr,
}

impl Config {
    pub fn validate(deps: Deps, input: InstantiateMsg) -> Result<Self, ContractError> {
        let requestor = RequestorConfig::validate(deps, input.requestor)?;
        let timeout = TimeoutConfig::validate(input.timeout)?;
        let verifier = deps.api.addr_validate(&input.verifier)?;
        Ok(Config {
            next_id: 1,
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

#[cw_serde]
pub struct TimeoutConfig {
    pub default: u64,
    pub minimum: u64,
    pub maximum: u64,
}

impl TimeoutConfig {
    pub fn validate(input: msg::TimeoutInfo) -> Result<Self, ContractError> {
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

    pub fn check_timeout(&self, timeout: Option<u64>) -> Result<u64, ContractError> {
        match timeout {
            Some(t) if t < self.minimum => Err(ContractError::TimeoutTooShort(self.minimum)),
            Some(t) if t > self.maximum => Err(ContractError::TimeoutTooLong(self.maximum)),
            Some(t) => Ok(t),
            None => Ok(self.default),
        }
    }
}

impl From<TimeoutConfig> for lavs_apis::tasks::TimeoutConfig {
    fn from(val: TimeoutConfig) -> Self {
        lavs_apis::tasks::TimeoutConfig {
            default: val.default,
            minimum: val.minimum,
            maximum: val.maximum,
        }
    }
}

#[cw_serde]
pub struct Task {
    pub description: String,
    pub status: Status,
    pub timing: Timing,
    pub payload: RequestType,
    pub result: Option<ResponseType>,
}

#[cw_serde]
pub struct Timing {
    /// Creation in UNIX seconds
    pub created_at: u64,
    /// Expiration in UNIX seconds
    pub expires_at: u64,
    /// Creation in block height
    pub created_height: u64,
}

impl Timing {
    pub fn new(env: &Env, timeout: u64) -> Self {
        Timing {
            created_at: env.block.time.seconds(),
            expires_at: env.block.time.seconds() + timeout,
            created_height: env.block.height,
        }
    }

    pub fn is_expired(&self, env: &Env) -> bool {
        self.expires_at <= env.block.time.seconds()
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