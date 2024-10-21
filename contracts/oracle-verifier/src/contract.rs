#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult,
};
use cw2::set_contract_version;
use lavs_apis::verifier_simple::OperatorVoteInfoResponse;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, SLASHED_OPERATORS, VOTES};

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let fields = [
        ("threshold_percentage", &msg.threshold_percentage),
        ("allowed_spread", &msg.allowed_spread),
        ("slashable_spread", &msg.slashable_spread),
    ];

    // checking if our fields are within valid 0..=100 bounds
    for (field_name, value) in fields.into_iter() {
        if *value == Decimal::zero() || value > &Decimal::percent(100) {
            return Err(ContractError::InvalidPercentage(
                field_name.to_string(),
                *value,
            ));
        }
    }

    if msg.slashable_spread <= msg.allowed_spread {
        return Err(ContractError::InvalidSpread(
            msg.slashable_spread,
            msg.allowed_spread,
        ));
    }
    let op_addr = deps.api.addr_validate(&msg.operator_contract)?;
    let config = Config {
        operator_contract: op_addr,
        threshold_percent: msg.threshold_percentage,
        allowed_spread: msg.allowed_spread,
        slashable_spread: msg.slashable_spread,
        required_percentage: msg.required_percentage,
    };

    CONFIG.save(deps.storage, &config)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ExecutedTask {
            task_queue_contract,
            task_id,
            result,
        } => execute::executed_task(deps, env, info, task_queue_contract, task_id, result),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let config = CONFIG.load(deps.storage)?;
            to_json_binary(&config)
        }
        QueryMsg::TaskInfo {
            task_contract,
            task_id,
        } => Ok(to_json_binary(&query::task_info(
            deps,
            env,
            task_contract,
            task_id,
        )?)?),
        QueryMsg::OperatorVote {
            task_contract,
            task_id,
            operator,
        } => Ok(to_json_binary(&query::query_operator_vote(
            deps,
            task_contract,
            task_id,
            operator,
        )?)?),
        QueryMsg::SlashableOperators {} => {
            let slashed_operators: Vec<Addr> = SLASHED_OPERATORS
                .keys(deps.storage, None, None, Order::Ascending)
                .collect::<StdResult<Vec<_>>>()?;
            to_json_binary(&slashed_operators)
        }
    }
}

mod execute {

    use cosmwasm_std::{to_json_binary, Decimal, Order, Uint128, WasmMsg};
    use cw_utils::nonpayable;
    use lavs_apis::{
        events::oracle_executed_event::{OracleExecutedEvent, OracleExecutionStatus},
        id::TaskId,
        tasks::{TaskExecuteMsg, TaskStatus},
    };
    use lavs_helpers::verifier::ensure_valid_vote;

    use crate::state::{record_vote, OperatorVote, SLASHED_OPERATORS, TASKS, VOTES};

    use super::*;

    pub fn executed_task(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        task_queue_contract: String,
        task_id: TaskId,
        result: String,
    ) -> Result<Response, ContractError> {
        nonpayable(&info)?;

        // validate task and operator
        let task_queue = deps.api.addr_validate(&task_queue_contract)?;
        let operator = info.sender;

        let config = CONFIG.load(deps.storage)?;

        // operator allowed to vote and hasn't voted yet
        let (mut task_data, power) = match ensure_valid_vote(
            deps.branch(),
            &env,
            &task_queue,
            task_id,
            &operator,
            config.required_percentage,
            &config.operator_contract,
        )? {
            Some(x) => x,
            None => return Ok(Response::default()),
        };

        // Update the vote and check the total power on this result, also recording the operators vote
        let tally = record_vote(
            deps.storage,
            &task_queue,
            task_id,
            &operator,
            &result,
            power,
        )?;

        let all_votes: Vec<(Addr, OperatorVote)> = VOTES
            .prefix((&task_queue, task_id))
            .range(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;

        let total_power: Uint128 = all_votes.iter().map(|(_, vote)| vote.power).sum();

        let mut resp = Response::new();

        if total_power < task_data.power_required {
            let event = OracleExecutedEvent {
                task_id,
                status: OracleExecutionStatus::VoteStored,
                new_price: None,
                task_queue_contract: task_queue_contract.clone(),
            };
            return Ok(resp.add_event(event));
        }

        let config = CONFIG.load(deps.storage)?;

        let (median, slashable_operators, is_threshold_met) =
            process_votes(&all_votes, tally, &config)?;

        if is_threshold_met {
            for operator in slashable_operators {
                noop_slash_validator(&mut deps, &operator)?;
            }

            task_data.status = TaskStatus::Completed;
            TASKS.save(deps.storage, (&task_queue, task_id), &task_data)?;

            let response = serde_json::json!(crate::state::PriceResult {
                price: median.to_string()
            });

            let msg = WasmMsg::Execute {
                contract_addr: task_queue.to_string(),
                msg: to_json_binary(&TaskExecuteMsg::Complete { task_id, response })?,
                funds: vec![],
            };

            resp = resp.add_message(msg);

            let event = OracleExecutedEvent {
                task_id,
                status: OracleExecutionStatus::ThresholdMet,
                new_price: Some(median),
                task_queue_contract: task_queue_contract.clone(),
            };

            resp = resp.add_event(event);
        } else {
            // this event and the one above can be DRY'ed, will leave it for later
            let event = OracleExecutedEvent {
                task_id,
                status: OracleExecutionStatus::ThresholdNotMet,
                new_price: None,
                task_queue_contract: task_queue_contract.clone(),
            };
            resp = resp.add_event(event);
        }

        // NOTE: If we ever want to optimize the storage we can remove the votes for the completed
        // tasks.

        Ok(resp)
    }

    pub(crate) fn calculate_median(values: &mut [Decimal]) -> Decimal {
        if values.is_empty() {
            return Decimal::zero();
        }

        values.sort();

        if values.len() % 2 == 0 {
            // first half                 + // second half              // divided by 2
            (values[values.len() / 2 - 1] + values[values.len() / 2]) / Uint128::new(2u128)
        } else {
            // take the middle value
            values[values.len() / 2]
        }
    }

    pub(crate) fn calculate_allowed_range(median: Decimal, spread: Decimal) -> (Decimal, Decimal) {
        let allowed_minimum = median * (Decimal::one() - spread);
        let allowed_maximum = median * (Decimal::one() + spread);
        (allowed_minimum, allowed_maximum)
    }

    pub(crate) fn filter_valid_votes(
        votes: &[(Addr, OperatorVote)],
        allowed_minimum_price: Decimal,
        allowed_maximum_price: Decimal,
    ) -> Vec<&(Addr, OperatorVote)> {
        votes
            .iter()
            .filter(|(_, vote)| {
                vote.result >= allowed_minimum_price && vote.result <= allowed_maximum_price
            })
            .collect()
    }

    pub(crate) fn is_threshold_met(
        valid_power: Uint128,
        total_power: Uint128,
        threshold_percent: Decimal,
    ) -> bool {
        let valid_ratio = Decimal::from_ratio(valid_power, total_power);
        valid_ratio >= threshold_percent
    }

    pub(crate) fn identify_slashable_operators(
        votes: &[(Addr, OperatorVote)],
        slashable_minimum: Decimal,
        slashable_maximum: Decimal,
    ) -> Vec<Addr> {
        votes
            .iter()
            .filter_map(|(operator_addr, vote)| {
                let price = vote.result;
                if price < slashable_minimum || price > slashable_maximum {
                    Some(operator_addr.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn noop_slash_validator(deps: &mut DepsMut, operator: &Addr) -> Result<(), ContractError> {
        SLASHED_OPERATORS.save(deps.storage, operator, &true)?;
        //TODO: this should make an actual call to slash
        Ok(())
    }

    pub(crate) fn process_votes(
        votes: &[(Addr, OperatorVote)],
        total_power: Uint128,
        config: &Config,
    ) -> Result<(Decimal, Vec<Addr>, bool), ContractError> {
        let mut all_prices: Vec<Decimal> = votes.iter().map(|(_, vote)| vote.result).collect();

        let median = calculate_median(&mut all_prices);

        let (allowed_minimum, allowed_maximum) =
            calculate_allowed_range(median, config.allowed_spread);

        let valid_votes = filter_valid_votes(votes, allowed_minimum, allowed_maximum);

        let valid_power: Uint128 = valid_votes.iter().map(|(_, vote)| vote.power).sum();

        let is_threshold_met = is_threshold_met(valid_power, total_power, config.threshold_percent);

        let (slashable_minimum, slashable_maximum) =
            calculate_allowed_range(median, config.slashable_spread);

        let slashable_operators =
            identify_slashable_operators(votes, slashable_minimum, slashable_maximum);

        Ok((median, slashable_operators, is_threshold_met))
    }
}

mod query {
    use lavs_apis::{
        id::TaskId,
        tasks::TaskStatus,
        verifier_simple::{TaskInfoResponse, TaskTally},
    };

    use crate::state::{OPTIONS, TASKS};

    use super::*;

    pub(crate) fn query_operator_vote(
        deps: Deps,
        task_contract: String,
        task_id: TaskId,
        operator: String,
    ) -> StdResult<Option<OperatorVoteInfoResponse>> {
        let task_contract = deps.api.addr_validate(&task_contract)?;
        let operator = deps.api.addr_validate(&operator)?;
        let vote = VOTES
            .may_load(deps.storage, (&task_contract, task_id, &operator))?
            .map(|v| OperatorVoteInfoResponse {
                power: v.power,
                result: v.result.to_string(),
            });
        Ok(vote)
    }

    pub fn task_info(
        deps: Deps,
        env: Env,
        task_contract: String,
        task_id: TaskId,
    ) -> StdResult<Option<TaskInfoResponse>> {
        let task_contract = deps.api.addr_validate(&task_contract)?;
        let info = TASKS.may_load(deps.storage, (&task_contract, task_id))?;
        if let Some(i) = info {
            // Check current time and update the status if it expired
            let status = match i.status {
                TaskStatus::Open if i.is_expired(&env) => TaskStatus::Expired,
                x => x,
            };
            // Collect the running tallies on the options
            let tallies: Result<Vec<_>, _> = OPTIONS
                .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
                .map(|r| {
                    r.map(|((_, _, result), v)| TaskTally {
                        result,
                        power: v.power,
                    })
                })
                .collect();
            let res = TaskInfoResponse {
                status,
                power_needed: i.power_required,
                tallies: tallies?,
            };
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state::OperatorVote;

    use super::*;
    use cosmwasm_std::{Decimal, Uint128};
    use execute::{
        calculate_allowed_range, calculate_median, filter_valid_votes,
        identify_slashable_operators, is_threshold_met, process_votes,
    };

    mod calculate_median {

        use super::*;

        #[test]
        fn calculate_median_odd_length() {
            let mut values = vec![Decimal::one(), Decimal::percent(300), Decimal::percent(500)];
            let median = calculate_median(&mut values);
            // we have 1, 3 and 5, so median should be 3
            assert_eq!(median, Decimal::percent(300));
        }

        #[test]
        fn calculate_median_even_length() {
            let mut values = vec![
                Decimal::one(),
                Decimal::percent(300),
                Decimal::percent(500),
                Decimal::percent(700),
            ];
            let median = calculate_median(&mut values);
            // this time we have 1, 3, 5 and 7 so median should be (3 + 5) / 2 = 4
            assert_eq!(median, Decimal::percent(400));
        }

        #[test]
        fn calculate_median_unsorted() {
            let mut values = vec![Decimal::percent(500), Decimal::one(), Decimal::percent(300)];
            let median = calculate_median(&mut values);
            // same as `calculate_median_odd_length` but unsorted
            assert_eq!(median, Decimal::percent(300));
        }

        #[test]
        fn calculate_median_single_element() {
            let mut values = vec![Decimal::percent(42)];
            let median = calculate_median(&mut values);
            assert_eq!(median, Decimal::percent(42));
        }

        #[test]
        fn calculate_median_fractional_values_odd() {
            let mut values = vec![
                Decimal::percent(11),
                Decimal::percent(12),
                Decimal::percent(13),
            ];
            let median = calculate_median(&mut values);
            // median should be 0.12
            assert_eq!(median, Decimal::percent(12));
        }

        #[test]
        fn calculate_median_fractional_values_even() {
            let mut values = vec![
                Decimal::percent(110),
                Decimal::percent(120),
                Decimal::percent(130),
                Decimal::percent(140),
            ];
            let median = calculate_median(&mut values);
            // (1.2 + 1.3) / 2 = 1.25
            assert_eq!(median, Decimal::percent(125));
        }

        #[test]
        fn calculate_median_identical_values() {
            let mut values = vec![
                Decimal::percent(500),
                Decimal::percent(500),
                Decimal::percent(500),
                Decimal::percent(500),
            ];
            let median = calculate_median(&mut values);
            assert_eq!(median, Decimal::percent(500));
        }

        #[test]
        fn calculate_median_large_numbers() {
            let mut values = vec![
                Decimal::percent(1_000_000_000_000u64),
                Decimal::percent(2_000_000_000_000u64),
                Decimal::percent(3_000_000_000_000u64),
            ];
            let median = calculate_median(&mut values);
            assert_eq!(median, Decimal::percent(2_000_000_000_000u64));
        }

        #[test]
        fn calculate_median_of_fib_unsorted() {
            let mut values = vec![
                Decimal::percent(340),
                Decimal::percent(20),
                Decimal::percent(550),
                Decimal::percent(50),
                Decimal::percent(80),
                Decimal::percent(130),
                Decimal::percent(30),
                Decimal::percent(210),
                Decimal::percent(1440),
                Decimal::percent(10),
                Decimal::percent(890),
                Decimal::percent(80),
            ];

            // This will be sorted to:
            // 0.10, 0.20, 0.30, 0.50, 0.80, 0.80, 1.30, 2.10, 3.40, 5.50, 8.90, 14.40
            let median = calculate_median(&mut values);
            assert_eq!(median, Decimal::percent(105)) // 1.05
        }

        #[test]
        fn calculate_median_empty() {
            let mut values: Vec<Decimal> = vec![];
            let median = calculate_median(&mut values);
            assert_eq!(median, Decimal::zero())
        }
    }

    mod allowed_range {
        use super::*;

        #[test]
        fn calculate_allowed_range_normal() {
            let median = Decimal::one();
            let spread = Decimal::percent(10);

            let (allowed_minimum, allowed_maximum) = calculate_allowed_range(median, spread);

            // allowed_minimum = 100 * (1 - 0.10) = 90
            // allowed_maximum = 100 * (1 + 0.10) = 110

            assert_eq!(allowed_minimum, Decimal::percent(90));
            assert_eq!(allowed_maximum, Decimal::percent(110));
        }

        #[test]
        fn calculate_allowed_range_zero_spread() {
            let median = Decimal::one();
            let spread = Decimal::zero();

            let (allowed_minimum, allowed_maximum) = calculate_allowed_range(median, spread);

            // allowed_minimum = 100 * (1 - 0) = 100
            // allowed_maximum = 100 * (1 + 0) = 100

            assert_eq!(allowed_minimum, median);
            assert_eq!(allowed_maximum, median);
        }

        #[test]
        fn calculate_allowed_range_full_spread() {
            let median = Decimal::one();
            let spread = Decimal::one();

            let (allowed_minimum, allowed_maximum) = calculate_allowed_range(median, spread);

            // allowed_minimum = 100 * (1 - 1) = 0
            // allowed_maximum = 100 * (1 + 1) = 200

            assert_eq!(allowed_minimum, Decimal::zero());
            assert_eq!(allowed_maximum, Decimal::percent(200));
        }

        #[test]
        fn calculate_allowed_range_zero_median() {
            let median = Decimal::zero();
            let spread = Decimal::percent(10);

            let (allowed_minimum, allowed_maximum) = calculate_allowed_range(median, spread);

            assert_eq!(allowed_minimum, Decimal::zero());
            assert_eq!(allowed_maximum, Decimal::zero());
        }

        #[test]
        fn calculate_allowed_range_fractional_median() {
            let median = Decimal::percent(1500);
            let spread = Decimal::percent(10);

            let (allowed_minimum, allowed_maximum) = calculate_allowed_range(median, spread);

            // allowed_minimum = 15.0 * (1 - 0.10) = 13.5
            // allowed_maximum = 15.0 * (1 + 0.10) = 16.5

            assert_eq!(allowed_minimum, Decimal::percent(1350)); // 13.5
            assert_eq!(allowed_maximum, Decimal::percent(1650)); // 16.5
        }

        #[test]
        fn calculate_allowed_range_fractional_spread() {
            let median = Decimal::one();
            // 0.15 or %15
            let spread = Decimal::percent(15);

            let (allowed_minimum, allowed_maximum) = calculate_allowed_range(median, spread);

            // allowed_minimum = 100 * (1 - 0.15) = 85
            // allowed_maximum = 100 * (1 + 0.15) = 115

            assert_eq!(allowed_minimum, Decimal::percent(85));
            assert_eq!(allowed_maximum, Decimal::percent(115));
        }

        #[test]
        fn calculate_allowed_range_large_numbers() {
            let median = Decimal::percent(1_000_000_000_000u64);
            let spread = Decimal::percent(10);

            let (allowed_minimum, allowed_maximum) = calculate_allowed_range(median, spread);

            // allowed_minimum = 1,000,000,000,000 * (1 - 0.1) = 900,000,000,000
            // allowed_maximum = 1,000,000,000,000 * (1 + 0.1) = 1,100,000,000,000

            assert_eq!(allowed_minimum, Decimal::percent(900_000_000_000u64));
            assert_eq!(allowed_maximum, Decimal::percent(1_100_000_000_000u64));
        }
    }

    mod filter_valid_votes {
        use super::*;

        #[test]
        fn filter_within_bounds() {
            let op1 = Addr::unchecked("addr1");
            let op2 = Addr::unchecked("addr2");
            let op3 = Addr::unchecked("addr3");

            let vote1 = OperatorVote {
                power: Uint128::new(100),
                result: Decimal::percent(150),
            };
            let vote2 = OperatorVote {
                power: Uint128::new(200),
                result: Decimal::percent(200),
            };
            let vote3 = OperatorVote {
                power: Uint128::new(300),
                result: Decimal::percent(250),
            };

            let votes = vec![
                (op1.clone(), vote1),
                (op2.clone(), vote2),
                (op3.clone(), vote3),
            ];

            // Allowed ranges
            let min_price = Decimal::percent(150);
            let max_price = Decimal::percent(250);

            let result = filter_valid_votes(&votes, min_price, max_price);

            assert_eq!(result.len(), 3);
            assert_eq!(
                result[0],
                &(
                    op1,
                    OperatorVote {
                        power: Uint128::new(100),
                        result: Decimal::percent(150)
                    }
                )
            );
            assert_eq!(
                result[1],
                &(
                    op2,
                    OperatorVote {
                        power: Uint128::new(200),
                        result: Decimal::percent(200)
                    }
                )
            );
            assert_eq!(
                result[2],
                &(
                    op3,
                    OperatorVote {
                        power: Uint128::new(300),
                        result: Decimal::percent(250)
                    }
                )
            );
        }

        #[test]
        fn filter_out_of_bounds() {
            let op1 = Addr::unchecked("addr1");
            let op2 = Addr::unchecked("addr2");
            let op3 = Addr::unchecked("addr3");

            let vote1 = OperatorVote {
                power: Uint128::new(100),
                result: Decimal::one(),
            };
            let vote2 = OperatorVote {
                power: Uint128::new(200),
                result: Decimal::percent(200),
            };
            let vote3 = OperatorVote {
                power: Uint128::new(300),
                result: Decimal::percent(300),
            };

            let votes = vec![
                (op1.clone(), vote1),
                (op2.clone(), vote2),
                (op3.clone(), vote3),
            ];

            let min_price = Decimal::percent(150);
            let max_price = Decimal::percent(250);

            let result = filter_valid_votes(&votes, min_price, max_price);

            assert_eq!(result.len(), 1);
            assert_eq!(
                result[0],
                &(
                    op2,
                    OperatorVote {
                        power: Uint128::new(200),
                        result: Decimal::percent(200)
                    }
                )
            );
        }

        #[test]
        fn filter_all_out_of_bounds() {
            let op1 = Addr::unchecked("addr1");
            let op2 = Addr::unchecked("addr2");
            let op3 = Addr::unchecked("addr3");

            let vote1 = OperatorVote {
                power: Uint128::new(100),
                result: Decimal::percent(50),
            };
            let vote2 = OperatorVote {
                power: Uint128::new(200),
                result: Decimal::percent(400),
            };
            let vote3 = OperatorVote {
                power: Uint128::new(300),
                result: Decimal::percent(500),
            };

            let votes = vec![
                (op1.clone(), vote1),
                (op2.clone(), vote2),
                (op3.clone(), vote3),
            ];

            let min_price = Decimal::percent(150);
            let max_price = Decimal::percent(250);

            let result = filter_valid_votes(&votes, min_price, max_price);

            assert_eq!(result.len(), 0);
        }

        #[test]
        fn filter_edge_cases() {
            let op1 = Addr::unchecked("addr1");
            let op2 = Addr::unchecked("addr2");

            let vote1 = OperatorVote {
                power: Uint128::new(100),
                result: Decimal::percent(150),
            };
            let vote2 = OperatorVote {
                power: Uint128::new(200),
                result: Decimal::percent(250),
            };

            let votes = vec![(op1.clone(), vote1), (op2.clone(), vote2)];

            let min_price = Decimal::percent(150);
            let max_price = Decimal::percent(250);

            let result = filter_valid_votes(&votes, min_price, max_price);

            assert_eq!(result.len(), 2);
            assert_eq!(
                result[0],
                &(
                    op1,
                    OperatorVote {
                        power: Uint128::new(100),
                        result: Decimal::percent(150)
                    }
                )
            );
            assert_eq!(
                result[1],
                &(
                    op2,
                    OperatorVote {
                        power: Uint128::new(200),
                        result: Decimal::percent(250)
                    }
                )
            );
        }
    }

    mod is_threshold_met {
        use super::*;

        #[test]
        fn threshold_met_exact() {
            let valid_power = Uint128::new(50);
            let total_power = Uint128::new(100);
            let threshold_percent = Decimal::percent(50);

            let result = is_threshold_met(valid_power, total_power, threshold_percent);
            assert!(
                result,
                "threshold should be met when valid is %50 of total power"
            );
        }

        #[test]
        fn threshold_not_met() {
            let valid_power = Uint128::new(40);
            let total_power = Uint128::new(100);
            let threshold_percent = Decimal::percent(50);

            let result = is_threshold_met(valid_power, total_power, threshold_percent);
            assert!(!result, "threshold should be not met when not enough power");
        }

        #[test]
        fn threshold_exceeded() {
            let valid_power = Uint128::new(60);
            let total_power = Uint128::new(100);
            let threshold_percent = Decimal::percent(50);

            let result = is_threshold_met(valid_power, total_power, threshold_percent);
            assert!(result, "should return true when threshold met over %50");
        }

        #[test]
        fn full_power_threshold() {
            let valid_power = Uint128::new(100);
            let total_power = Uint128::new(100);
            let threshold_percent = Decimal::one();

            let result = is_threshold_met(valid_power, total_power, threshold_percent);
            assert!(
                result,
                "should return true when valid power is equal total power"
            );
        }

        #[test]
        fn threshold_met_minimum_case() {
            let valid_power = Uint128::new(2);
            let total_power = Uint128::new(100);
            let threshold_percent = Decimal::percent(1);

            let result = is_threshold_met(valid_power, total_power, threshold_percent);
            assert!(
                result,
                "should return true when valid power is over the threshold"
            );
        }
    }

    mod identify_slashable_operators {
        use super::*;

        #[test]
        fn no_slashable_operators() {
            let op1 = Addr::unchecked("operator1");
            let op2 = Addr::unchecked("operator2");
            let op3 = Addr::unchecked("operator3");

            let vote1 = OperatorVote {
                power: Uint128::new(100),
                result: Decimal::percent(150),
            };
            let vote2 = OperatorVote {
                power: Uint128::new(200),
                result: Decimal::percent(200),
            };
            let vote3 = OperatorVote {
                power: Uint128::new(300),
                result: Decimal::percent(250),
            };

            let votes = vec![
                (op1.clone(), vote1),
                (op2.clone(), vote2),
                (op3.clone(), vote3),
            ];

            let slashable_minimum = Decimal::percent(150);
            let slashable_maximum = Decimal::percent(250);

            let result = identify_slashable_operators(&votes, slashable_minimum, slashable_maximum);
            assert_eq!(result.len(), 0, "there should be no slashable operators");
        }

        #[test]
        fn some_slashable_operators() {
            let op1 = Addr::unchecked("operator1");
            let op2 = Addr::unchecked("operator2");
            let op3 = Addr::unchecked("operator3");

            let vote1 = OperatorVote {
                power: Uint128::new(100),
                result: Decimal::one(),
            };
            let vote2 = OperatorVote {
                power: Uint128::new(200),
                result: Decimal::percent(200),
            };
            let vote3 = OperatorVote {
                power: Uint128::new(300),
                result: Decimal::percent(300),
            };

            let votes = vec![
                (op1.clone(), vote1),
                (op2.clone(), vote2),
                (op3.clone(), vote3),
            ];

            let slashable_minimum = Decimal::percent(150);
            let slashable_maximum = Decimal::percent(250);

            let result = identify_slashable_operators(&votes, slashable_minimum, slashable_maximum);
            assert_eq!(result.len(), 2, "we must have 2 slashable operators");
            assert_eq!(result, vec![op1.clone(), op3.clone()]);
        }

        #[test]
        fn all_slashable_operators() {
            let op1 = Addr::unchecked("operator1");
            let op2 = Addr::unchecked("operator2");
            let op3 = Addr::unchecked("operator3");

            let vote1 = OperatorVote {
                power: Uint128::new(100),
                result: Decimal::percent(50),
            };
            let vote2 = OperatorVote {
                power: Uint128::new(200),
                result: Decimal::percent(300),
            };
            let vote3 = OperatorVote {
                power: Uint128::new(300),
                result: Decimal::percent(400),
            };

            let votes = vec![
                (op1.clone(), vote1),
                (op2.clone(), vote2),
                (op3.clone(), vote3),
            ];

            let slashable_minimum = Decimal::percent(150);
            let slashable_maximum = Decimal::percent(250);

            let result = identify_slashable_operators(&votes, slashable_minimum, slashable_maximum);
            assert_eq!(result.len(), 3, "all operators should be slashed");
            assert_eq!(result, vec![op1.clone(), op2.clone(), op3.clone()]);
        }

        #[test]
        fn edge_case_slashable_operators() {
            let op1 = Addr::unchecked("operator1");
            let op2 = Addr::unchecked("operator2");

            let vote1 = OperatorVote {
                power: Uint128::new(100),
                // low blound
                result: Decimal::percent(150),
            };
            let vote2 = OperatorVote {
                power: Uint128::new(200),
                // upper bound
                result: Decimal::percent(250),
            };

            let votes = vec![(op1.clone(), vote1), (op2.clone(), vote2)];

            let slashable_minimum = Decimal::percent(150);
            let slashable_maximum = Decimal::percent(250);

            let result = identify_slashable_operators(&votes, slashable_minimum, slashable_maximum);
            assert_eq!(result.len(), 0, "operators shouldn't be slashed");
        }

        #[test]
        fn empty_votes() {
            let votes: Vec<(Addr, OperatorVote)> = vec![];

            let slashable_minimum = Decimal::percent(150);
            let slashable_maximum = Decimal::percent(250);

            let result = identify_slashable_operators(&votes, slashable_minimum, slashable_maximum);
            assert_eq!(result.len(), 0, "there should be none from an empty list");
        }
    }

    mod process_votes {
        use super::*;

        #[test]
        fn process_votes_meets_threshold() {
            let op1 = Addr::unchecked("operator1");
            let op2 = Addr::unchecked("operator2");

            let votes = vec![
                (
                    op1.clone(),
                    OperatorVote {
                        power: Uint128::new(100),
                        result: Decimal::one(),
                    },
                ),
                (
                    op2.clone(),
                    OperatorVote {
                        power: Uint128::new(100),
                        result: Decimal::one(),
                    },
                ),
            ];

            let config = Config {
                operator_contract: Addr::unchecked("operators"),
                threshold_percent: Decimal::percent(50),
                allowed_spread: Decimal::percent(10),
                slashable_spread: Decimal::percent(20),
                required_percentage: 70,
            };

            // mocking the power
            let result = process_votes(&votes, Uint128::new(100), &config).unwrap();

            let expected_median = Decimal::one();
            let expected_slashable_operators: Vec<Addr> = vec![];
            let expected_is_threshold_met = true;

            assert_eq!(result.0, expected_median);
            assert_eq!(result.1, expected_slashable_operators);
            assert_eq!(result.2, expected_is_threshold_met);
        }

        #[test]
        fn process_votes_threshold_not_met() {
            let op1 = Addr::unchecked("operator1");
            let op2 = Addr::unchecked("operator2");

            let votes = vec![
                (
                    op1.clone(),
                    OperatorVote {
                        power: Uint128::new(20),
                        result: Decimal::one(),
                    },
                ),
                (
                    op2.clone(),
                    OperatorVote {
                        power: Uint128::new(90),
                        result: Decimal::percent(300),
                    },
                ),
            ];

            let config = Config {
                operator_contract: Addr::unchecked("operators"),
                threshold_percent: Decimal::percent(80),
                allowed_spread: Decimal::percent(10),
                slashable_spread: Decimal::percent(20),
                required_percentage: 70,
            };

            // mocking the power
            let result = process_votes(&votes, Uint128::new(100), &config).unwrap();

            let expected_median = Decimal::percent(200);
            let expected_slashable_operators = vec![op1.clone(), op2.clone()];
            let expected_is_threshold_met = false;

            assert_eq!(result.0, expected_median);
            assert_eq!(result.1, expected_slashable_operators);
            assert_eq!(result.2, expected_is_threshold_met);
        }

        #[test]
        fn test_process_votes_slashable_operators() {
            let op1 = Addr::unchecked("operator1");
            let op2 = Addr::unchecked("operator2");
            let op3 = Addr::unchecked("operator3");

            let votes = vec![
                (
                    op1.clone(),
                    OperatorVote {
                        power: Uint128::new(50),
                        result: Decimal::percent(150),
                    },
                ),
                (
                    op2.clone(),
                    OperatorVote {
                        power: Uint128::new(50),
                        result: Decimal::percent(200),
                    },
                ),
                (
                    op3.clone(),
                    OperatorVote {
                        power: Uint128::new(50),
                        result: Decimal::percent(350),
                    },
                ),
            ];

            let config = Config {
                operator_contract: Addr::unchecked("operators"),
                threshold_percent: Decimal::percent(33),
                allowed_spread: Decimal::percent(10),
                slashable_spread: Decimal::percent(20),
                required_percentage: 70,
            };

            // mocking the power
            let result = process_votes(&votes, Uint128::new(100), &config).unwrap();

            let expected_median = Decimal::percent(200);
            let expected_slashable_operators = vec![op1.clone(), op3.clone()];
            let expected_is_threshold_met = true;

            assert_eq!(result.0, expected_median);
            assert_eq!(result.1, expected_slashable_operators);
            assert_eq!(result.2, expected_is_threshold_met);
        }

        #[test]
        fn test_process_votes_insufficient_power_votes() {
            let operator1 = Addr::unchecked("operator1");
            let operator2 = Addr::unchecked("operator2");
            let operator3 = Addr::unchecked("operator3");

            let total_power = Uint128::new(100);

            let config = Config {
                operator_contract: Addr::unchecked("operator_contract"),
                threshold_percent: Decimal::percent(50),
                allowed_spread: Decimal::percent(10),
                slashable_spread: Decimal::percent(20),
                required_percentage: 70,
            };

            // submitted are 100.00 and 102.00
            let votes = vec![
                (
                    operator1.clone(),
                    OperatorVote {
                        result: Decimal::percent(10000),
                        power: Uint128::new(20),
                    },
                ),
                (
                    operator2.clone(),
                    OperatorVote {
                        result: Decimal::percent(10200),
                        power: Uint128::new(20),
                    },
                ),
            ];

            let (median, slashed_operators, is_threshold_met) =
                process_votes(&votes, total_power, &config).unwrap();

            assert!(!is_threshold_met);
            // 101
            assert_eq!(median, Decimal::percent(10100));
            assert_eq!(slashed_operators.len(), 0);

            let votes_with_op3 = vec![
                votes[0].clone(),
                votes[1].clone(),
                (
                    operator3.clone(),
                    OperatorVote {
                        // 98
                        result: Decimal::percent(9800),
                        power: Uint128::new(60),
                    },
                ),
            ];

            let (median, slashed_operators, is_threshold_met) =
                process_votes(&votes_with_op3, total_power, &config).unwrap();

            assert!(is_threshold_met);
            // 100
            assert_eq!(median, Decimal::percent(10000));
            assert_eq!(slashed_operators.len(), 0);
        }

        #[test]
        fn test_process_votes_spread_exceeds_allowed() {
            let operator1 = Addr::unchecked("operator1");
            let operator2 = Addr::unchecked("operator2");
            let operator3 = Addr::unchecked("operator3");

            let total_power = Uint128::new(100);

            let config = Config {
                operator_contract: Addr::unchecked("operator_contract"),
                threshold_percent: Decimal::one(),
                allowed_spread: Decimal::percent(10),
                slashable_spread: Decimal::percent(20),
                required_percentage: 70,
            };

            // submited are 1.0 1.3 and 0.7
            let votes = vec![
                (
                    operator1.clone(),
                    OperatorVote {
                        result: Decimal::percent(100),
                        power: Uint128::new(50),
                    },
                ),
                (
                    operator2.clone(),
                    OperatorVote {
                        result: Decimal::percent(130),
                        power: Uint128::new(30),
                    },
                ),
                (
                    operator3.clone(),
                    OperatorVote {
                        result: Decimal::percent(70),
                        power: Uint128::new(20),
                    },
                ),
            ];

            let (median, slashed_operators, is_threshold_met) =
                process_votes(&votes, total_power, &config).unwrap();

            assert!(!is_threshold_met);
            assert_eq!(median, Decimal::percent(100));
            assert_eq!(slashed_operators.len(), 2);
        }

        #[test]
        fn test_process_votes_one_operator_slashed() {
            let operator1 = Addr::unchecked("operator1");
            let operator2 = Addr::unchecked("operator2");
            let operator3 = Addr::unchecked("operator3");

            let total_power = Uint128::new(100);

            let config = Config {
                operator_contract: Addr::unchecked("operator_contract"),
                threshold_percent: Decimal::percent(80),
                allowed_spread: Decimal::percent(10),
                slashable_spread: Decimal::percent(20),
                required_percentage: 70,
            };

            // submited are 1.0 1.05 and 1.5
            let votes = vec![
                (
                    operator1.clone(),
                    OperatorVote {
                        result: Decimal::percent(100),
                        power: Uint128::new(50),
                    },
                ),
                (
                    operator2.clone(),
                    OperatorVote {
                        result: Decimal::percent(105),
                        power: Uint128::new(30),
                    },
                ),
                (
                    operator3.clone(),
                    OperatorVote {
                        // Outlier
                        result: Decimal::percent(150),
                        power: Uint128::new(20),
                    },
                ),
            ];

            let (median, slashed_operators, is_threshold_met) =
                process_votes(&votes, total_power, &config).unwrap();

            assert!(is_threshold_met);
            assert_eq!(median, Decimal::percent(105));
            assert_eq!(slashed_operators, vec![operator3.clone()]);
        }

        #[test]
        fn test_process_votes_median_calculation() {
            let operator1 = Addr::unchecked("operator1");
            let operator2 = Addr::unchecked("operator2");
            let operator3 = Addr::unchecked("operator3");

            let total_power = Uint128::new(100);

            let config = Config {
                operator_contract: Addr::unchecked("operator_contract"),
                threshold_percent: Decimal::one(),
                allowed_spread: Decimal::percent(50),
                slashable_spread: Decimal::percent(60),
                required_percentage: 70,
            };

            // submitted are 1.0 1.1 and 1.2
            let votes = vec![
                (
                    operator1.clone(),
                    OperatorVote {
                        result: Decimal::percent(100),
                        power: Uint128::new(50),
                    },
                ),
                (
                    operator2.clone(),
                    OperatorVote {
                        result: Decimal::percent(110),
                        power: Uint128::new(30),
                    },
                ),
                (
                    operator3.clone(),
                    OperatorVote {
                        result: Decimal::percent(120),
                        power: Uint128::new(20),
                    },
                ),
            ];

            let (median, slashed_operators, is_threshold_met) =
                process_votes(&votes, total_power, &config).unwrap();

            assert_eq!(median, Decimal::percent(110));
            assert!(is_threshold_met);
            assert_eq!(slashed_operators.len(), 0);
        }
    }
}
