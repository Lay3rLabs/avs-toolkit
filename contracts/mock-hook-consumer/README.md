# Mock Hook Consumer Contract

This contract serves as a mock implementation of a hook consumer for task hooks in a CosmWasm environment. It demonstrates how to handle various task-related events and showcases different response patterns for each event type.

## Overview

The mock hook consumer contract responds to three types of task events:

1. Task Created
2. Task Completed
3. Task Timeout

Each event is handled differently to showcase various patterns and possibilities when working with task hooks.

## Usage

### Instantiation

The contract is instantiated with an empty message:

```rust
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response>
```

### Execution

The contract handles `TaskHook` messages, which can be one of the following:

- `TaskCreatedHook(task)`
- `TaskCompletedHook(task)`
- `TaskTimeoutHook(task)`

### Task Flow

1. When a task is created, the contract increments its created counter.
2. When a task is completed, the contract:
   - Deserializes the task response
   - Creates a new task request (squaring the result)
   - Sends the new task to the task queue
3. When a task times out, the contract intentionally throws an error.