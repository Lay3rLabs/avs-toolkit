use cosmwasm_std::Timestamp;
use cw_orch::environment::{ChainState, CwEnv};
use cw_orch::prelude::*;
use lavs_apis::events::task_executed_event::TaskExecutedEvent;
use lavs_apis::id::TaskId;
use lavs_apis::time::Duration;
use serde_json::json;

use lavs_apis::tasks::{Requestor, Status, TaskStatus, TimeoutInfo};
use lavs_orch::{Addressable, AltSigner};

use lavs_mock_operators::interface::Contract as MockOperatorsContract;
use lavs_mock_operators::msg::{
    InstantiateMsg as MockOperatorsInstantiateMsg, InstantiateOperator,
    QueryMsgFns as MockOperatorsQueryMsgFns,
};
use lavs_task_queue::interface::Contract as TasksContract;
use lavs_task_queue::msg::{
    CustomExecuteMsgFns as TasksExecuteMsgFns, CustomQueryMsgFns as TasksQueryMsgFns,
    InstantiateMsg as TasksInstantiateMsg,
};

use crate::interface::Contract;
use crate::msg::{ExecuteMsgFns, InstantiateMsg, QueryMsgFns};

pub const BECH_PREFIX: &str = "layer";

pub fn setup<Chain: CwEnv>(chain: Chain, msg: InstantiateMsg) -> Contract<Chain> {
    let contract = Contract::new(chain);
    contract.upload().unwrap();
    contract.instantiate(&msg, None, &[]).unwrap();
    contract
}

/// Some basic tests to show it works. Setup and signing
pub fn happy_path<C>(chain: C)
where
    C: CwEnv + AltSigner,
    C::Sender: Addressable,
{
    // This is the node who can submit to the validator
    let op_node = chain.alt_signer(3);
    // Random person who cannot approve
    let invalid_op_node = chain.alt_signer(4);

    // Upload and instantiate operator contract with one operator
    let operators = vec![InstantiateOperator::new(op_node.addr().to_string(), 20)];
    let msg = MockOperatorsInstantiateMsg { operators };
    let operators = MockOperatorsContract::new(chain.clone());
    operators.upload().unwrap();
    operators.instantiate(&msg, None, &[]).unwrap();

    // Upload and instantiate verifier, connecting to the operator
    let msg = InstantiateMsg {
        operator_contract: operators.addr_str().unwrap(),
        required_percentage: 70,
    };
    let verifier = setup(chain.clone(), msg);

    // Upload and instantiate task queue, acknowledging the verifier
    let msg = TasksInstantiateMsg {
        requestor: Requestor::Fixed(chain.sender_addr().into()),
        timeout: TimeoutInfo::new(Duration::new_seconds(600)),
        verifier: verifier.addr_str().unwrap(),
        owner: None,
        task_specific_whitelist: None,
    };
    let tasker = TasksContract::new(chain.clone());
    tasker.upload().unwrap();
    tasker.instantiate(&msg, None, &[]).unwrap();

    // Create a task
    let payload = json!({"x": 17});
    let task_id = make_task(&tasker, "Test Task", None, &payload);

    // Random cannot verify
    let bad_result = r#"{"y": 33}"#.to_string();
    let _ = verifier
        .call_as(&invalid_op_node)
        .executed_task(tasker.addr_str().unwrap(), task_id, bad_result)
        .unwrap_err();

    // Operator can verify
    let result = r#"{"y": 289}"#.to_string();
    let call_result = verifier
        .call_as(&op_node)
        .executed_task(tasker.addr_str().unwrap(), task_id, result)
        .unwrap();

    let event = call_result
        .events()
        .iter()
        .find_map(|event| TaskExecutedEvent::try_from(event).ok())
        .unwrap();

    assert!(event.completed);
    assert_eq!(event.task_id, task_id);
    assert_eq!(event.task_queue, tasker.addr_str().unwrap());
    assert_eq!(event.operator, op_node.addr().to_string());

    let completed = chain.block_info().unwrap().time;

    // Check it is marked as completed (both in verifier and task queue)
    let status = tasker.task(task_id).unwrap();
    assert_eq!(status.status, Status::Completed { completed });
    assert_eq!(status.result, Some(json!({"y": 289})));

    let v_status = verifier
        .task_info(tasker.addr_str().unwrap(), task_id)
        .unwrap();
    assert_eq!(v_status.unwrap().status, TaskStatus::Completed);
}

/// Ensure a 2 of 3 needs 2 to pass
pub fn require_quorum<C>(chain: C)
where
    C: CwEnv + AltSigner,
    C::Sender: Addressable,
{
    // This is the node who can submit to the validator
    let op_nodes = [
        chain.alt_signer(3),
        chain.alt_signer(4),
        chain.alt_signer(5),
    ];

    // Upload and instantiate operator contract with one operator
    let operators = op_nodes
        .iter()
        .map(|n| InstantiateOperator::new(n.addr().to_string(), 1))
        .collect();
    let msg = MockOperatorsInstantiateMsg { operators };
    let operators = MockOperatorsContract::new(chain.clone());
    operators.upload().unwrap();
    operators.instantiate(&msg, None, &[]).unwrap();

    // Upload and instantiate verifier, connecting to the operator
    let msg = InstantiateMsg {
        operator_contract: operators.addr_str().unwrap(),
        required_percentage: 65, // 65% of 3 means 2 needed
    };
    let verifier = setup(chain.clone(), msg);

    // Upload and instantiate task queue, acknowledging the verifier
    let msg = TasksInstantiateMsg {
        requestor: Requestor::Fixed(chain.sender_addr().into()),
        timeout: TimeoutInfo::new(Duration::new_seconds(600)),
        verifier: verifier.addr_str().unwrap(),
        owner: None,
        task_specific_whitelist: None,
    };
    let tasker = TasksContract::new(chain.clone());
    tasker.upload().unwrap();
    tasker.instantiate(&msg, None, &[]).unwrap();

    // check the operator config
    let total_power = operators.total_power_at_height(None).unwrap();
    assert_eq!(total_power.power.u128(), 3u128);

    let one_power = operators
        .voting_power_at_height(op_nodes[0].addr().into_string(), None)
        .unwrap();
    assert_eq!(one_power.power.u128(), 1u128);

    let all_voters = operators.all_voters().unwrap();
    assert_eq!(all_voters.voters.len(), 3);

    // Create a task
    let payload = json!({"x": 17});
    let task_id = make_task(&tasker, "Test Task", None, &payload);

    // Operator 1 can vote on it
    let result = r#"{"y": 289}"#.to_string();
    verifier
        .call_as(&op_nodes[0])
        .executed_task(tasker.addr_str().unwrap(), task_id, result)
        .unwrap();
    // But not yet completed
    let status = tasker.task(task_id).unwrap();
    assert!(matches!(status.status, Status::Open { .. }));

    // Operator 2 votes on different item, not completed
    let result = r#"{"y": 291}"#.to_string();
    verifier
        .call_as(&op_nodes[1])
        .executed_task(tasker.addr_str().unwrap(), task_id, result)
        .unwrap();
    // But not yet completed
    let status = tasker.task(task_id).unwrap();
    assert!(matches!(status.status, Status::Open { .. }));

    // Operator 1 cannot vote twice
    let result = r#"{"y": 222}"#.to_string();
    let _ = verifier
        .call_as(&op_nodes[0])
        .executed_task(tasker.addr_str().unwrap(), task_id, result)
        .unwrap_err();

    // Operator 3 vote agrees with 1 and marks it as completed
    let result = r#"{"y": 289}"#.to_string();
    verifier
        .call_as(&op_nodes[2])
        .executed_task(tasker.addr_str().unwrap(), task_id, result)
        .unwrap();
    let completed = Timestamp::from_nanos(chain.block_info().unwrap().time.nanos());

    // Check it is marked as completed (both in verifier and task queue)
    let status = tasker.task(task_id).unwrap();
    assert_eq!(status.status, Status::Completed { completed });
    assert_eq!(status.result, Some(json!({"y": 289})));

    let v_status = verifier
        .task_info(tasker.addr_str().unwrap(), task_id)
        .unwrap();
    assert_eq!(v_status.unwrap().status, TaskStatus::Completed);
}

// TODO: add a

#[track_caller]
pub fn make_task<C: ChainState + TxHandler>(
    contract: &TasksContract<C>,
    name: &str,
    timeout: impl Into<Option<Duration>>,
    payload: &serde_json::Value,
) -> TaskId {
    let res = contract
        .create(
            name.to_string(),
            timeout.into(),
            payload.clone(),
            None,
            None,
            &[],
        )
        .unwrap();
    get_task_id(&res)
}

// Note: return types for methods depends on the chain...
// mock -> abstract_cw_multi_test::AppResponse
// osmosis test tube -> abstract_cw_multi_test::AppResponse
// daemon -> cw_orch::daemon::CosmTxResponse
//
// Note: both implement cw_orch::environment::IndexResponse
#[track_caller]
pub fn get_task_id(res: &impl IndexResponse) -> TaskId {
    res.event_attr_value("wasm-task_created_event", "task-id")
        .unwrap()
        .parse()
        .unwrap()
}
