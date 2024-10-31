# Lay3r AVS Task Queue

This is a simple contract to maintain a task queue for use with an AVS. This will become
quite configurable over time, the general flow is:

- Create Task
- Complete Task
- Timeout

## Configuration

The following configuration settings will be exposed:

Requestor - an enum of:

- Fixed Address (one address that can request)
- Open Payment (any address with min fee)

Verifier: Address of another contract that will verify any results and is the only address that
can mark a request completed, along with the verified result.

Min, Default, Max Timeout: The minimum, maximum, and default values of task timeouts. If the task creator
doesn't provide a value, we will use the default. Otherwise, we assert the user-provided value is in the
proscribed range.

## Actions

### Create Task

This will be configurable to either one address that can create tasks (add to the queue),
or a minimum fee. If the fee is set, anyone can add a task by paying the fee.

### Complete Task

Anyone can submit a proposed response to the verifier contract to complete a task. This will perform custom
logic to ensure correctness and then call the task queue if it passes.

### Timeout Task

Anyone can call to mark a task as timed out if the block time has passed the task-specified timeout.

## Queries

- List all tasks (most recently created first)
- List open tasks (most recently created first)
- List closed tasks (most recently created first)
- Get Task info by id (included status and result if any)

## Data

Tasks will have `RequestData` as part of the structure and the verifiers will write `ResponseData`
upon a successful validation. Both of these should be generics to allow the implementation of
the task queue to define the specific types it supports.

For MVP, we will use `serde::Value` to allow any arbitrary JSON for request and response. Later,
we want to make this more specific.

## Cw Orch Powered Testing

Run the test cases on a real network.

1. `(sudo) ./scripts/optimize.sh`
2. `cd contracts/avs/tasks`
3. `cargo run --example devnet --features dev`

Ensure you have set `TEST_MNEMONIC` in `contracts/avs/tasks/.env` to an account with some tokens.

## Task Hooks

The contract supports hooks to notify other contracts about task-related events. This enables automated workflows and reactions to task lifecycle changes.

### Hook Types

- `Created`: Triggered when a new task is created
- `Completed`: Triggered when a task is successfully completed
- `Timeout`: Triggered when a task times out

### Hook Scopes

Hooks can be configured at two levels:
- **Global Hooks**: Apply to all tasks in the system
- **Task-Specific Hooks**: Apply only to individual tasks

### Access Control

- **Global Hooks**: Can only be managed by the contract owner
- **Task-Specific Hooks**: Can be managed by:
  - The contract owner
  - Addresses in the task-specific whitelist (for their own tasks)

### Task-Specific Whitelist

The contract maintains a whitelist of addresses that can create task-specific hooks. This whitelist is managed by the contract owner and allows for delegated hook management while maintaining security.

### Hook Messages

When triggered, hooks send messages to their registered receiver contracts containing relevant task information such as:
- Task ID
- Event type (Created/Completed/Timeout)
- Task status
- Timestamp
- Additional context (e.g., result data for completed tasks)

The receiver contract must implement the appropriate message handling for these hook notifications: [Example](../mock-hook-consumer/README.md###Execution).

## TODO

We have a working MVP but need to make some improvements for this to be production-ready.
Capturing a basic list of missing features here (bug fixes should be issues):

- Efficient implementation of the list queries
- Data cleanup (what tasks can be deleted? expired? completed after X second?)
- More tests
- Deployment script (along with other contracts)
