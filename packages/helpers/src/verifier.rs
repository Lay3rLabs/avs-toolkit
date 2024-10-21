use cosmwasm_std::{Addr, DepsMut, Env, Uint128};
use lavs_apis::{
    id::TaskId,
    interfaces::{tasks::TasksStorage, voting::VotingPowerResponse},
    verifier_simple::{TaskMetadata, VerifierError},
};

use lavs_apis::interfaces::voting::QueryMsg as OperatorQueryMsg;

/// Does all checks to ensure the voter is valid and has not voted yet.
/// Also checks the task is valid and still open.
/// Returns the metadata for the task (creating it if first voter), along with the voting power of this operator.
///
/// We do not want to error if an operator votes for a task that is already completed (due to race conditions).
/// In that case, just return None and exit early rather than error.
#[allow(clippy::too_many_arguments)]
pub fn ensure_valid_vote(
    mut deps: DepsMut,
    env: &Env,
    task_queue: &Addr,
    task_id: TaskId,
    operator: &Addr,
    fraction_percent: u32,
    operators_addr: &Addr,
) -> Result<Option<(TaskMetadata, Uint128)>, VerifierError> {
    // Load task info, or create it if not there
    // Error here means the contract is in expired or completed, return None rather than error
    let metadata = match TasksStorage::<'_>::handle_metadata(
        deps.branch(),
        env,
        operators_addr,
        task_queue,
        task_id,
        fraction_percent,
    ) {
        Ok(x) => x,
        Err(_) => return Ok(None),
    };

    // Get the operators voting power at time of vote creation
    let power: VotingPowerResponse = deps.querier.query_wasm_smart(
        operators_addr.to_string(),
        &OperatorQueryMsg::VotingPowerAtHeight {
            address: operator.to_string(),
            height: Some(metadata.created_height),
        },
    )?;
    if power.power.is_zero() {
        return Err(VerifierError::Unauthorized);
    }

    Ok(Some((metadata, power.power)))
}
