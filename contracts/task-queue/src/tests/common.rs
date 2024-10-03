use cw_orch::environment::{ChainState, CwEnv, Environment, IndexResponse, QueryHandler};
use cw_orch::prelude::*;
use lavs_apis::id::TaskId;
use lavs_apis::tasks::TaskStatus;
use serde_json::json;

use crate::error::ContractError;
use crate::interface::Contract as TaskContract;
use crate::msg::{
    CompletedTaskOverview, InstantiateMsg, ListCompletedResponse, ListOpenResponse,
    OpenTaskOverview, Requestor, Status, TimeoutInfo,
};

// FIXME: any way to get these as one import, rather than import all sub traits?
// Maybe combining them in a super-trait?
use crate::msg::{CustomExecuteMsgFns, CustomQueryMsgFns, TaskExecuteMsgFns, TaskQueryMsgFns};

use lavs_orch::{Addressable, AltSigner};

const VERIFIER_INDEX: u32 = 3;

pub fn setup<Chain: CwEnv>(chain: Chain, msg: InstantiateMsg) -> TaskContract<Chain> {
    let tasker = TaskContract::new(chain);
    tasker.upload().unwrap();
    tasker.instantiate(&msg, None, &[]).unwrap();
    tasker
}

fn fixed_requestor<Chain>(chain: &Chain, timeout: u64) -> (TaskContract<Chain>, Chain::Sender)
where
    Chain: CwEnv + AltSigner,
    Chain::Sender: Addressable,
{
    let verifier = chain.alt_signer(VERIFIER_INDEX);
    let msg = InstantiateMsg {
        requestor: Requestor::Fixed(chain.sender_addr().into()),
        timeout: mock_timeout(timeout),
        verifier: verifier.addr().into(),
    };

    let contract = setup(chain.clone(), msg);
    (contract, verifier)
}

// This sets the chain signer to be the requestor
// Pass another verifier in
// (not sure how to generically get Addr from C::Sender for both mock and daemon)
pub fn happy_path<C>(chain: C)
where
    C: CwEnv + AltSigner,
    C::Sender: Addressable,
{
    let (contract, verifier) = fixed_requestor(&chain, 200);

    contract.code_id().expect("Code not uploaded");
    contract.address().expect("Contract not instantiated");

    let payload_one = json! ({ "pair": ["eth", "usd"]});
    let payload_two = json! ({ "pair": ["btc", "usd"]});

    let result = json!({ "result": "success" });

    // create two tasks
    let one = make_task(&contract, "One", 300, &payload_one);
    let start = get_time(contract.environment());
    contract.environment().next_block().unwrap();
    let two = make_task(&contract, "Two", 100, &payload_two);
    let start_two = get_time(contract.environment());
    contract.environment().next_block().unwrap();
    assert_ne!(start, start_two);
    assert_eq!(two, TaskId::new(one.u64() + 1));

    // query for open tasks
    let open = contract.list_open(None, None).unwrap();
    assert_eq!(open.tasks.len(), 2);
    let closed = contract.list_completed(None, None).unwrap();
    assert_eq!(closed.tasks.len(), 0);

    // fail to verify one task
    let err = contract.complete(one, result.clone()).unwrap_err();
    println!("Bad verfier error: {:?}", err);

    // complete one task
    contract
        .call_as(&verifier)
        .complete(one, result.clone())
        .unwrap();
    let end = get_time(contract.environment());

    // check the list queries
    let open = contract.list_open(None, None).unwrap();
    assert_eq!(open.tasks.len(), 1);
    let closed = contract.list_completed(None, None).unwrap();
    assert_eq!(closed.tasks.len(), 1);

    // check the details of the completed task
    let details = contract.task(one).unwrap();
    // We need to allow a little leeway, as start and end times may be off by a second
    // Thus no equals comparision, but the destructuring test on status
    assert_eq!(details.id, one);
    assert_eq!(details.description, "One".to_string());
    assert_eq!(details.payload, payload_one);
    assert_eq!(details.result, Some(result));
    match details.status {
        Status::Completed { completed } => {
            assert!(
                completed >= end - 2 && completed <= end,
                "completed: {}, end: {}",
                completed,
                end
            );
        }
        _ => panic!("unexpected status"),
    }
}

pub fn create<C>(chain: C)
where
    C: CwEnv + AltSigner,
    C::Sender: Addressable,
{
    let (contract, verifier) = fixed_requestor(&chain, 200);
    let payload = json! ({ "pair": ["eth", "usd"]});

    let config = contract.config().unwrap();
    assert_eq!(config.timeout.default, 200);
    assert_eq!(config.timeout.minimum, 100);
    assert_eq!(config.timeout.maximum, 400);
    assert_eq!(
        config.requestor,
        Requestor::Fixed(chain.sender_addr().into())
    );
    assert_eq!(config.verifier, verifier.addr().into_string());

    // Note: you need the root error to get that from the contract.
    // {} will just show the method call,
    // {:#} or {:?} will show the full error chain (but :# is nicer to read)
    let err = contract
        .create("Too Short".to_string(), Some(4), payload.clone(), &[])
        .unwrap_err();
    assert!(
        err.root()
            .to_string()
            .contains(&ContractError::TimeoutTooShort(100).to_string()),
        "Unexpected error: {}",
        err.root()
    );

    let one = contract
        .create("One".to_string(), None, payload.clone(), &[])
        .unwrap();
    let task_one = one.event_attr_value("wasm", "task_id").unwrap();
    assert_eq!(task_one, "1");
    let task_one: u64 = task_one.parse().unwrap();
    assert_eq!(task_one, 1u64);

    let two = contract
        .create("Two".to_string(), None, payload.clone(), &[])
        .unwrap();
    let task_two = get_task_id(&two);
    assert_eq!(task_two, TaskId::new(2u64));
}

// These measurements are a bit different between multi-test and golem.
// Multi-test doesn't move a block until explicitly told, so only counts the next_blocks
// Golem moves forward one block on each execution, so this is double the time in this case.
// Also, the time to the contract is *after* the block moves forward.
// Thus, we set offset to one block time, and block_time to 2 block times...
pub fn list_open_queries_with_expiration<C>(chain: C, block_time: u64, offset: u64)
where
    C: CwEnv + AltSigner,
    C::Sender: Addressable,
{
    let (contract, _verifier) = fixed_requestor(&chain, 200);
    let payload_one = json! ({ "pair": ["eth", "usd"]});
    let payload_two = json! ({ "pair": ["btc", "usd"]});
    let payload_three = json! ({ "pair": ["atom", "eur"]});

    let start = chain.block_info().unwrap().time.seconds();
    let one = make_task(&contract, "One", 300, &payload_one);
    chain.next_block().unwrap();
    let two = make_task(&contract, "Two", 100, &payload_two);
    chain.next_block().unwrap();
    let three = make_task(&contract, "Two", None, &payload_three); // uses default of 200

    let ListOpenResponse { tasks } = contract.list_open(None, None).unwrap();
    assert_eq!(tasks.len(), 3);
    assert_eq!(
        tasks[0],
        OpenTaskOverview {
            id: three,
            expires: start + 200 + 2 * block_time + offset, // we waited two blocks to create
            payload: payload_three,
        }
    );
    assert_eq!(
        tasks[1],
        OpenTaskOverview {
            id: two,
            expires: start + 100 + block_time + offset, // we waited one block to create
            payload: payload_two,
        }
    );
    assert_eq!(
        tasks[2],
        OpenTaskOverview {
            id: one,
            expires: start + 300 + offset,
            payload: payload_one,
        }
    );

    // now let's wait a bit so some expire
    chain.wait_seconds(150).unwrap();
    let ListOpenResponse { tasks } = contract.list_open(None, None).unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].id, three);
    assert_eq!(tasks[1].id, one);

    // and the next expiration
    chain.wait_seconds(100).unwrap();
    let ListOpenResponse { tasks } = contract.list_open(None, None).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, one);

    // and them all
    chain.wait_seconds(100).unwrap();
    let ListOpenResponse { tasks } = contract.list_open(None, None).unwrap();
    assert_eq!(tasks.len(), 0);
}

pub fn completion_works<C>(chain: C)
where
    C: CwEnv + AltSigner,
    C::Sender: Addressable,
{
    let (contract, verifier) = fixed_requestor(&chain, 200);
    let payload = json! ({ "pair": ["eth", "usd"]});
    let result = json! ({ "price": "1234.56"});

    let one = make_task(&contract, "One", 300, &payload);
    chain.next_block().unwrap();
    let two = make_task(&contract, "Two", 100, &payload);

    // list completed empty
    let ListCompletedResponse { tasks } = contract.list_completed(None, None).unwrap();
    assert_eq!(tasks.len(), 0);

    // normal user cannot complete
    let err = contract.complete(one, result.clone()).unwrap_err();
    assert!(
        err.root()
            .to_string()
            .contains(&ContractError::Unauthorized.to_string()),
        "unexpected error: {}",
        err.root(),
    );

    // verifier can complete
    contract
        .call_as(&verifier)
        .complete(one, result.clone())
        .unwrap();
    let completion_time = chain.block_info().unwrap().time.seconds();
    chain.next_block().unwrap();

    // cannot complete already completed
    let err = contract
        .call_as(&verifier)
        .complete(one, result.clone())
        .unwrap_err();
    assert!(
        err.root()
            .to_string()
            .contains(&ContractError::TaskCompleted.to_string()),
        "unexpected error: {}",
        err.root(),
    );

    // cannot complete unknown task ids
    let err = contract
        .call_as(&verifier)
        .complete(TaskId::new(two.u64() + 1), result.clone())
        .unwrap_err();
    assert!(err.root().to_string().contains("not found"));

    // cannot complete expired
    chain.wait_seconds(100).unwrap();
    let err = contract
        .call_as(&verifier)
        .complete(two, result.clone())
        .unwrap_err();
    assert!(
        err.root()
            .to_string()
            .contains(&ContractError::TaskExpired.to_string()),
        "unexpected error: {}",
        err.root(),
    );

    // list completed
    let ListCompletedResponse { tasks } = contract.list_completed(None, None).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(
        tasks[0],
        CompletedTaskOverview {
            id: one,
            completed: completion_time,
            result,
        }
    );
}

pub fn task_status_works<C>(chain: C)
where
    C: CwEnv + AltSigner,
    C::Sender: Addressable,
{
    let (contract, verifier) = fixed_requestor(&chain, 200);
    let payload = json! ({ "pair": ["eth", "usd"]});
    let result = json! ({ "price": "1234.56"});

    let one = make_task(&contract, "One", 300, &payload);
    chain.next_block().unwrap();
    let two = make_task(&contract, "Two", 100, &payload);

    // check open status
    let status_one = contract.task_status(one).unwrap();
    let status_two = contract.task_status(two).unwrap();
    assert_eq!(status_one.created_time + 300, status_one.expires_time);
    assert_eq!(status_two.created_time + 100, status_two.expires_time);
    assert!(status_one.created_height < status_two.created_height);
    assert_eq!(status_one.status, TaskStatus::Open);
    assert_eq!(status_two.status, TaskStatus::Open);

    // verifier can complete
    contract
        .call_as(&verifier)
        .complete(one, result.clone())
        .unwrap();
    chain.next_block().unwrap();

    // contract one changed
    let status_one = contract.task_status(one).unwrap();
    assert_eq!(status_one.status, TaskStatus::Completed);
    // contract two unchanged
    let status_two = contract.task_status(two).unwrap();
    assert_eq!(status_two.status, TaskStatus::Open);

    // expried contracts automatically update return value
    chain.wait_seconds(200).unwrap();
    // contract one unchanged
    let status_one = contract.task_status(one).unwrap();
    assert_eq!(status_one.status, TaskStatus::Completed);
    // contract two expired
    let status_two = contract.task_status(two).unwrap();
    assert_eq!(status_two.status, TaskStatus::Expired);
}

pub fn task_pagination_works<C>(chain: C)
where
    C: CwEnv + AltSigner,
    C::Sender: Addressable,
{
    let (contract, _verifier) = fixed_requestor(&chain, 1000);
    let payload = json!({"pair": ["eth", "usd"]});

    // Create 10 tasks with increasing timeouts
    let mut created_tasks = Vec::new();
    for i in 1..=10 {
        let task_id = make_task(
            &contract,
            &format!("Task {}", i),
            Some(1000 + i * 100),
            &payload,
        );
        created_tasks.push(task_id);
        chain.next_block().unwrap();
    }

    // Get the total number of open tasks
    let ListOpenResponse {
        tasks: all_open_tasks,
    } = contract.list_open(None, None).unwrap();
    let total_open_tasks = all_open_tasks.len();

    // Test pagination with different limits
    let test_cases = vec![2, 3, 5, 7, 10, 15];

    for limit in test_cases {
        let mut all_retrieved_tasks = Vec::new();
        let mut start_after = None;

        loop {
            let ListOpenResponse { tasks } = contract.list_open(start_after, Some(limit)).unwrap();

            if tasks.is_empty() {
                break;
            }

            // Check that each page has the correct number of tasks
            assert!(tasks.len() <= limit as usize, "Page size exceeds limit");

            // Check that there's no overlap with previously retrieved tasks
            for task in &tasks {
                assert!(
                    !all_retrieved_tasks.contains(&task.id),
                    "Task {} appeared in multiple pages",
                    task.id
                );
            }

            // If it's not the first page, check that the first task of this page has an older task id.
            // Newest tasks are retrieved first.
            if let Some(last_task_id) = start_after {
                assert!(
                    tasks[0].id < last_task_id,
                    "First task of new page ({:?}) should have older task id ({:?})",
                    tasks[0].id,
                    last_task_id
                );
            }

            all_retrieved_tasks.extend(tasks.iter().map(|t| t.id));
            start_after = tasks.last().map(|t| t.id);
        }

        // Check total number of tasks retrieved
        assert_eq!(
            all_retrieved_tasks.len(),
            total_open_tasks,
            "Number of tasks retrieved ({}) doesn't match total open tasks ({})",
            all_retrieved_tasks.len(),
            total_open_tasks
        );

        // Check that all created tasks (that are still open) are in the retrieved tasks
        for task_id in &created_tasks {
            if all_open_tasks.iter().any(|t| t.id == *task_id) {
                assert!(
                    all_retrieved_tasks.contains(task_id),
                    "Created task {:?} is missing from retrieved tasks",
                    task_id
                );
            }
        }
    }

    // Test with no limit (should return all open tasks)
    let ListOpenResponse { tasks } = contract.list_open(None, None).unwrap();
    assert_eq!(
        tasks.len(),
        total_open_tasks,
        "Should return all open tasks when no limit is specified"
    );
}

#[track_caller]
pub fn get_time(chain: &impl QueryHandler) -> u64 {
    chain.block_info().unwrap().time.seconds()
}

#[track_caller]
pub fn make_task<C: ChainState + TxHandler>(
    contract: &TaskContract<C>,
    name: &str,
    timeout: impl Into<Option<u64>>,
    payload: &serde_json::Value,
) -> TaskId {
    let res = contract
        .create(name.to_string(), timeout.into(), payload.clone(), &[])
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
    res.event_attr_value("wasm", "task_id")
        .unwrap()
        .parse()
        .unwrap()
}

// sets up a range around 50% to 200% of the default timeout
pub fn mock_timeout(default: u64) -> TimeoutInfo {
    TimeoutInfo {
        default,
        minimum: Some(default / 2),
        maximum: Some(default * 2),
    }
}
