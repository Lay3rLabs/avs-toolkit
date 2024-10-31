use cosmwasm_std::{Timestamp, Uint128};
use cw_orch::environment::{ChainState, CwEnv, Environment, IndexResponse, QueryHandler};
use cw_orch::prelude::*;
use lavs_apis::id::TaskId;
use lavs_apis::interfaces::task_hooks::TaskHookType;
use lavs_apis::tasks::{InfoStatus, TaskInfoResponse, TaskStatus};
use lavs_apis::time::Duration;
use mock_hook_consumer::msg::{ExecuteMsgFns, QueryMsgFns as _};
use serde_json::json;

use crate::error::ContractError;
use crate::interface::Contract as TaskContract;
use crate::msg::{
    CompletedTaskOverview, InstantiateMsg, ListCompletedResponse, ListOpenResponse, ListResponse,
    OpenTaskOverview, Requestor, Status, TimeoutInfo,
};
use crate::tests::multi::DENOM;
use mock_hook_consumer::interface::Contract as MockHookConsumerContract;

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

pub fn setup_mock_hooks_consumer<Chain: CwEnv>(chain: Chain) -> MockHookConsumerContract<Chain> {
    let hook_consumer = MockHookConsumerContract::new(chain);
    hook_consumer.upload().unwrap();
    hook_consumer.instantiate(&Empty {}, None, &[]).unwrap();
    hook_consumer
}

fn fixed_requestor<Chain>(chain: &Chain, timeout: Duration) -> (TaskContract<Chain>, Chain::Sender)
where
    Chain: CwEnv + AltSigner,
    Chain::Sender: Addressable,
{
    let verifier = chain.alt_signer(VERIFIER_INDEX);
    let msg = InstantiateMsg {
        requestor: Requestor::Fixed(chain.sender_addr().into()),
        timeout: mock_timeout(timeout),
        verifier: verifier.addr().into(),
        owner: None,
        task_specific_whitelist: None,
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
    let (contract, verifier) = fixed_requestor(&chain, Duration::new_seconds(200));

    contract.code_id().expect("Code not uploaded");
    contract.address().expect("Contract not instantiated");

    let payload_one = json! ({ "pair": ["eth", "usd"]});
    let payload_two = json! ({ "pair": ["btc", "usd"]});

    let result = json!({ "result": "success" });

    // create two tasks
    let one = make_task(
        &contract,
        "One",
        Some(Duration::new_seconds(300)),
        &payload_one,
    );
    let start = get_time(contract.environment());
    contract.environment().next_block().unwrap();
    let two = make_task(
        &contract,
        "Two",
        Some(Duration::new_seconds(100)),
        &payload_two,
    );
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
                completed >= end.minus_seconds(2) && completed <= end,
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
    let (contract, verifier) = fixed_requestor(&chain, Duration::new_seconds(200));
    let payload = json! ({ "pair": ["eth", "usd"]});

    let config = contract.config().unwrap();
    assert_eq!(config.timeout.default, Duration::new_seconds(200));
    assert_eq!(config.timeout.minimum, Duration::new_seconds(100));
    assert_eq!(config.timeout.maximum, Duration::new_seconds(400));
    assert_eq!(
        config.requestor,
        Requestor::Fixed(chain.sender_addr().into())
    );
    assert_eq!(config.verifier, verifier.addr().into_string());

    // Note: you need the root error to get that from the contract.
    // {} will just show the method call,
    // {:#} or {:?} will show the full error chain (but :# is nicer to read)
    let err = contract
        .create(
            "Too Short".to_string(),
            Some(Duration::new_seconds(4)),
            payload.clone(),
            None,
            None,
            &[],
        )
        .unwrap_err();
    assert!(
        err.root()
            .to_string()
            .contains(&ContractError::TimeoutTooShort(Duration::new_seconds(100)).to_string()),
        "Unexpected error: {}",
        err.root()
    );

    let one = contract
        .create("One".to_string(), None, payload.clone(), None, None, &[])
        .unwrap();
    let task_one = one
        .event_attr_value("wasm-task_created_event", "task-id")
        .unwrap();
    assert_eq!(task_one, "1");
    let task_one: u64 = task_one.parse().unwrap();
    assert_eq!(task_one, 1u64);

    let two = contract
        .create("Two".to_string(), None, payload.clone(), None, None, &[])
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
    let (contract, _verifier) = fixed_requestor(&chain, Duration::new_seconds(200));
    let payload_one = json! ({ "pair": ["eth", "usd"]});
    let payload_two = json! ({ "pair": ["btc", "usd"]});
    let payload_three = json! ({ "pair": ["atom", "eur"]});

    let start = chain.block_info().unwrap().time;
    let one = make_task(
        &contract,
        "One",
        Some(Duration::new_seconds(300)),
        &payload_one,
    );
    chain.next_block().unwrap();
    let two = make_task(
        &contract,
        "Two",
        Some(Duration::new_seconds(100)),
        &payload_two,
    );
    chain.next_block().unwrap();
    let three = make_task(&contract, "Two", None, &payload_three); // uses default of 200

    let ListOpenResponse { tasks } = contract.list_open(None, None).unwrap();
    assert_eq!(tasks.len(), 3);

    let ListResponse { tasks: all_tasks } = contract.list(None, None).unwrap();
    assert_eq!(all_tasks.len(), 3);

    let calculate_expiration =
        |start: Timestamp, n_blocks: u64, timeout_seconds: u64| -> Timestamp {
            let duration = Duration::new_seconds(n_blocks * block_time + offset + timeout_seconds);
            Timestamp::from_nanos(start.nanos() + duration.as_nanos())
        };

    let task_three_expiration = calculate_expiration(start, 2, 200);

    assert_eq!(
        tasks[0],
        OpenTaskOverview {
            id: three,
            expires: task_three_expiration, // we waited two blocks to create
            payload: payload_three,
        }
    );

    let task_two_expiration = calculate_expiration(start, 1, 100);
    assert_eq!(
        tasks[1],
        OpenTaskOverview {
            id: two,
            expires: task_two_expiration, // we waited one block to create
            payload: payload_two,
        }
    );

    let task_one_expiration = calculate_expiration(start, 0, 300);
    assert_eq!(
        tasks[2],
        OpenTaskOverview {
            id: one,
            expires: task_one_expiration,
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

    let ListResponse { tasks: all_tasks } = contract.list(None, None).unwrap();
    assert_eq!(all_tasks.len(), 3);
}

pub fn completion_works<C>(chain: C)
where
    C: CwEnv + AltSigner,
    C::Sender: Addressable,
{
    let (contract, verifier) = fixed_requestor(&chain, Duration::new_seconds(200));
    let payload = json! ({ "pair": ["eth", "usd"]});
    let result = json! ({ "price": "1234.56"});

    let one_created = chain.block_info().unwrap().time;
    let one = make_task(&contract, "One", Some(Duration::new_seconds(300)), &payload);

    chain.next_block().unwrap();

    let two_created = chain.block_info().unwrap().time;
    let two = make_task(&contract, "Two", Some(Duration::new_seconds(100)), &payload);

    // list completed empty
    let ListCompletedResponse { tasks } = contract.list_completed(None, None).unwrap();
    assert_eq!(tasks.len(), 0);

    // two total tasks exist
    let ListResponse { tasks: all_tasks } = contract.list(None, None).unwrap();
    assert_eq!(all_tasks.len(), 2);
    assert_eq!(
        all_tasks,
        vec![
            TaskInfoResponse {
                id: two,
                description: "Two".to_string(),
                status: InfoStatus::Open {
                    expires: two_created.plus_seconds(100)
                },
                payload: payload.clone(),
                result: None,
                created_at: two_created,
            },
            TaskInfoResponse {
                id: one,
                description: "One".to_string(),
                status: InfoStatus::Open {
                    expires: one_created.plus_seconds(300)
                },
                payload: payload.clone(),
                result: None,
                created_at: one_created,
            },
        ]
    );

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
    let completion_time = Timestamp::from_nanos(chain.block_info().unwrap().time.nanos());
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
            result: result.clone(),
        }
    );

    // two total tasks still exist
    let ListResponse { tasks: all_tasks } = contract.list(None, None).unwrap();
    assert_eq!(all_tasks.len(), 2);
    assert_eq!(
        all_tasks,
        vec![
            TaskInfoResponse {
                id: two,
                description: "Two".to_string(),
                status: InfoStatus::Expired {
                    expired: two_created.plus_seconds(100)
                },
                payload: payload.clone(),
                result: None,
                created_at: two_created,
            },
            TaskInfoResponse {
                id: one,
                description: "One".to_string(),
                status: InfoStatus::Completed {
                    completed: completion_time
                },
                payload,
                result: Some(result),
                created_at: one_created,
            },
        ]
    );
}

pub fn task_status_works<C>(chain: C)
where
    C: CwEnv + AltSigner,
    C::Sender: Addressable,
{
    let (contract, verifier) = fixed_requestor(&chain, Duration::new_seconds(200));
    let payload = json! ({ "pair": ["eth", "usd"]});
    let result = json! ({ "price": "1234.56"});

    let one = make_task(&contract, "One", Some(Duration::new_seconds(300)), &payload);
    chain.next_block().unwrap();
    let two = make_task(&contract, "Two", Some(Duration::new_seconds(100)), &payload);

    // check open status
    let status_one = contract.task_status(one).unwrap();
    let status_two = contract.task_status(two).unwrap();

    assert_eq!(
        status_one.created_time.plus_seconds(300),
        status_one.expires_time
    );
    assert_eq!(
        status_two.created_time.plus_seconds(100),
        status_two.expires_time
    );
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
    let (contract, _verifier) = fixed_requestor(&chain, Duration::new_seconds(1000));
    let payload = json!({"pair": ["eth", "usd"]});

    // Create 10 tasks with increasing timeouts
    let mut created_tasks = Vec::new();
    for i in 1..=10 {
        let task_id = make_task(
            &contract,
            &format!("Task {}", i),
            Some(Duration::new_seconds(1000 + i * 100)),
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

pub fn mock_hook_consumer_test<C>(chain: C, mock_consumer: MockHookConsumerContract<C>)
where
    C: CwEnv + AltSigner,
    C::Sender: Addressable,
{
    let coin = Coin {
        denom: DENOM.to_string(),
        amount: Uint128::one(),
    };
    let funds = vec![coin.clone()];

    // Setup the task contract
    let verifier = chain.alt_signer(VERIFIER_INDEX);
    let msg = InstantiateMsg {
        requestor: Requestor::OpenPayment(coin),
        timeout: mock_timeout(Duration::new_seconds(200)),
        verifier: verifier.addr().into(),
        owner: None, // defaults to sender
        task_specific_whitelist: Some(vec![mock_consumer.addr_str().unwrap()]),
    };
    let task_contract = setup(chain.clone(), msg);

    // Establish hooks
    let task_id_for_specific_hook = TaskId::new(1);
    task_contract
        .add_hooks(
            None,
            TaskHookType::Created,
            vec![mock_consumer.addr_str().unwrap()],
        )
        .unwrap();
    task_contract
        .add_hooks(
            Some(task_id_for_specific_hook), // Only task 1 will create another task on completion
            TaskHookType::Completed,
            vec![mock_consumer.addr_str().unwrap()],
        )
        .unwrap();
    task_contract
        .add_hooks(
            None,
            TaskHookType::Timeout,
            vec![mock_consumer.addr_str().unwrap()],
        )
        .unwrap();

    // Ensure the task-specific hook count is 1
    let task_hooks = task_contract
        .task_hooks(TaskHookType::Completed, Some(task_id_for_specific_hook))
        .unwrap();
    assert_eq!(task_hooks.hooks.len(), 1);

    // Ensure created counter starts at 0
    let counter = mock_consumer.created_count().unwrap();
    assert!(counter == 0);

    // Create a task
    let payload = json!({"x": 5});
    let task_id = make_task_with_funds(&task_contract, "Test Task", None, &payload, &funds);

    // Verify task created hook
    let counter = mock_consumer.created_count().unwrap();
    assert_eq!(counter, 1);

    // Complete the task
    let result = json!({"y": 25});
    task_contract
        .call_as(&verifier)
        .complete(task_id, result.clone())
        .unwrap();

    // Verify task completed hook and new task creation
    let new_task_id = TaskId::new(task_id.u64() + 1);
    let new_task = task_contract.task(new_task_id).unwrap();
    assert_eq!(new_task.description, "Test Task");
    assert_eq!(new_task.payload, json!({"x": 25}));

    // Create a task that will timeout
    let timeout_task_id = make_task_with_funds(
        &task_contract,
        "Timeout Task",
        Some(Duration::new_seconds(100)),
        &payload,
        &funds,
    );

    // Wait for the task to expire
    chain.wait_seconds(150).unwrap();

    // Timeout the task
    // The consumer should error out, but the function should not block
    task_contract
        .call_as(&verifier)
        .timeout(timeout_task_id)
        .unwrap();

    // Create another task
    let payload = json!({"x": 5});
    let task_id = make_task_with_funds(&task_contract, "Test Task", None, &payload, &funds);

    // Complete this task
    let result = json!({"y": 25});
    task_contract
        .call_as(&verifier)
        .complete(task_id, result)
        .unwrap();

    // Ensure task count is only 4
    // The task-specific hook only created another task for task 1
    let task_list = task_contract.list(None, None).unwrap();
    assert_eq!(task_list.tasks.len(), 4);

    // Ensure the task-specific hook was removed
    let task_hooks = task_contract
        .task_hooks(TaskHookType::Completed, Some(task_id_for_specific_hook))
        .unwrap();
    assert!(task_hooks.hooks.is_empty());

    // Register a hook from the mock-hook-consumer to test the task-specific whitelist (task that was created from mock_consumer)
    mock_consumer
        .register_hook(TaskHookType::Completed, new_task_id)
        .unwrap();

    // Other user can't register a hook on that same task
    task_contract
        .call_as(&chain.alt_signer(1))
        .add_hooks(
            Some(new_task_id),
            TaskHookType::Completed,
            vec![mock_consumer.addr_str().unwrap()],
        )
        .unwrap_err();

    // Process this task
    let result = json!({"y": 625});
    task_contract
        .call_as(&verifier)
        .complete(new_task_id, result)
        .unwrap();

    // Ensure another task was created - 5
    let task_list = task_contract.list(None, None).unwrap();
    assert_eq!(task_list.tasks.len(), 5);

    // Update task-specific whitelist
    let whitelisted = chain.alt_signer(1);
    task_contract
        .update_task_specific_whitelist(Some(vec![whitelisted.addr().to_string()]), None)
        .unwrap();

    // Non-whitelisted account cannot create a task with atomic task hooks
    task_contract
        .call_as(&chain.alt_signer(2))
        .create(
            "Task with unauthorized atomic task hooks",
            None,
            payload.clone(),
            None,
            Some(vec![mock_consumer.addr_str().unwrap()]),
            &funds,
        )
        .unwrap_err();

    // Test atomic hooks
    let task_id = TaskId::new(6);
    task_contract
        .call_as(&whitelisted)
        .create(
            "Task with atomic task hooks",
            None,
            payload.clone(),
            None,
            Some(vec![mock_consumer.addr_str().unwrap()]),
            &funds,
        )
        .unwrap();

    // Complete this task
    let result = json!({"y": 25});
    task_contract
        .call_as(&verifier)
        .complete(task_id, result)
        .unwrap();

    // Ensure another task was created on top of this - 7
    let task_list = task_contract.list(None, None).unwrap();
    assert_eq!(task_list.tasks.len(), 7);
}

pub fn timeout_refund_test<C>(chain: C, denom: String)
where
    C: CwEnv + AltSigner,
    C::Sender: Addressable,
{
    // Setup
    let coin = Coin {
        denom,
        amount: Uint128::new(100),
    };
    let funds = vec![coin.clone()];

    let verifier = chain.alt_signer(VERIFIER_INDEX);
    let msg = InstantiateMsg {
        requestor: Requestor::OpenPayment(coin.clone()),
        timeout: mock_timeout(Duration::new_seconds(200)),
        verifier: verifier.addr().into(),
        owner: None,
        task_specific_whitelist: None,
    };
    let task_contract = setup(chain.clone(), msg);

    // Get initial balance
    let binding = chain
        .balance(&chain.sender_addr(), Some(coin.denom.clone()))
        .unwrap();
    let initial_balance = binding.first().unwrap();

    // Create a task
    let payload = json!({"test": "data"});
    let task_id = make_task_with_funds(
        &task_contract,
        "Refund Test",
        Some(Duration::new_seconds(100)),
        &payload,
        &funds,
    );

    // Check balance after task creation
    let binding = chain
        .balance(&chain.sender_addr(), Some(coin.denom.clone()))
        .unwrap();
    let balance_after_creation = binding.first().unwrap();
    assert_eq!(
        balance_after_creation.amount,
        initial_balance.amount - coin.amount,
        "Balance should decrease by task cost after creation"
    );

    // Wait for the task to expire
    chain.wait_seconds(200).unwrap();

    // Timeout the task
    task_contract.call_as(&verifier).timeout(task_id).unwrap();

    // Check balance after timeout
    let binding = chain
        .balance(&chain.sender_addr(), Some(coin.denom.clone()))
        .unwrap();
    let balance_after_timeout = binding.first().unwrap();
    assert_eq!(
        balance_after_timeout, initial_balance,
        "Balance should be refunded to initial amount after timeout"
    );

    // Verify task status
    let task_status = task_contract.task_status(task_id).unwrap();
    assert_eq!(
        task_status.status,
        TaskStatus::Expired,
        "Task status should be Expired after timeout"
    );
}

#[track_caller]
pub fn get_time(chain: &impl QueryHandler) -> Timestamp {
    chain.block_info().unwrap().time
}

#[track_caller]
pub fn make_task<C: ChainState + TxHandler>(
    contract: &TaskContract<C>,
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

#[track_caller]
pub fn make_task_with_funds<C: ChainState + TxHandler>(
    contract: &TaskContract<C>,
    name: &str,
    timeout: impl Into<Option<Duration>>,
    payload: &serde_json::Value,
    funds: &[Coin],
) -> TaskId {
    let res = contract
        .create(
            name.to_string(),
            timeout.into(),
            payload.clone(),
            None,
            None,
            funds,
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

// sets up a range around 50% to 200% of the default timeout
pub fn mock_timeout(default: Duration) -> TimeoutInfo {
    TimeoutInfo {
        default,
        minimum: Some(Duration::new_seconds(default.as_seconds() / 2)),
        maximum: Some(Duration::new_seconds(default.as_seconds() * 2)),
    }
}
