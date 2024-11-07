# Demo with Testnet

This shows how to deploy an AVS on the permissionless and setup stuff

## Check Testnet Connectivity

```bash
# Ensure the chain is up
curl https://rpc.hack.layer.xyz/status

# see blocks are being made.. run a few times
curl https://rpc.hack.layer.xyz/status | jq .result.sync_info

# ensure wasmatic opertators are set up with different addresses
curl https://op1.hack.layer.xyz/info | jq -r ".operators[0]"
curl https://op2.hack.layer.xyz/info | jq -r ".operators[0]"
curl https://op3.hack.layer.xyz/info | jq -r ".operators[0]"
```

## Deploy AVS stuff

Go into `tools/cli` dir to run all the other commands.
More info available on the [README there](./tools/cli/README.md).

### Set Up Wallet

```bash
M=$(cargo run wallet create | tail -1 | cut -c16-)
echo "TEST_MNEMONIC=\"$M\"" > .env
# this should have a nice 24 word phrase
cat .env

cargo run faucet tap
cargo run wallet show
```

### Deploy Contracts

```bash
# rebuild the contracts locally
(cd ../.. && ./scripts/optimizer.sh)

# deploy them
cargo run deploy --mode verifier-simple contracts --operators wasmatic

# Copy the line that says "export TEST_TASK_QUEUE_ADDRESS" and paste it in your shell

cargo run task-queue view-queue
```

### Deploy WASI component

```bash
# setup wasi
(cd ../.. && ./scripts/setup_wasi.sh)

# rebuild the component
(cd ../.. && ./scripts/build_wasi.sh)

# Testable is optional if you want to try the next step
# Do not use for production deployments
cargo run -- wasmatic deploy --name YOUR_NAME_HERE \
    --wasm-source ../../components/cavs_square.wasm  \
    --testable \
    --task $TEST_TASK_QUEUE_ADDRESS
```

## Test a Component

This can only be done if `--testable` was provided above

```bash
cargo run -- wasmatic test --name YOUR_NAME_HERE --input '{"x": 32}'
```

It will parse the input as if you pushed it to the task queue and return
the result (or error) to the caller. Nothing is written on chain.

Note: if you change state when being triggered, this will break the AVS
consensus mechanism (different results for different operators), and thus
should not be used in production.

## Trigger Task

```bash
cargo run task-queue view-queue

cargo run task-queue add-task -b '{"x": 12}' -d 'test 1'

# wait a few secords, or until the log output shows it is executed
cargo run task-queue view-queue
```

### Task Hooks

Task hooks allow contracts to receive notifications for task events. The task queue's owner can manage hooks for any receiver address, with the option to make hooks either global or specific to particular tasks. The task owner can allow task creators to add hooks to their own tasks through the task-specific whitelist.

View current hooks:
```bash
# View global hooks for a specific type
cargo run task-queue view-hooks --hook-type completed

# View hooks for specific type and task
cargo run task-queue view-hooks --hook-type completed --task-id TASK_ID

# Available hook types: completed, timeout, created
```

Manage task-specific whitelist:
```bash
# Add users to the task-specific whitelist
cargo run task-queue update-task-specific-whitelist --to-add ADDRESS_1,ADDRESS_2,ADDRESS_3

# Remove users from the task-specific whitelist
cargo run task-queue update-task-specific-whitelist --to-remove ADDRESS_1,ADDRESS_2

# View the task-specific whitelist
cargo run task-queue view-task-specific-whitelist
```

Register a hook for task events:
```bash
# Add global hooks for completed tasks
cargo run task-queue add-hooks \
    --hook-type completed \
    --receivers CONTRACT_ADDRESS_1,CONTRACT_ADDRESS_2

# Add task-specific hooks
cargo run task-queue add-hooks \
    --hook-type completed \
    --receivers CONTRACT_ADDRESS \
    --task-id TASK_ID

# Add hooks for timeouts
cargo run task-queue add-hooks \
    --hook-type timeout \
    --receivers CONTRACT_ADDRESS_HERE \
    [--task-id TASK_ID]

# Add hooks for future tasks
cargo run task-queue add-hooks \
    --hook-type created \
    --receivers CONTRACT_ADDRESS_HERE \
    [--task-id TASK_ID]
```

Remove hooks when no longer needed:
```bash
# Remove global hook
cargo run task-queue remove-hook \
    --hook-type completed \
    --receiver CONTRACT_ADDRESS_HERE

# Remove task-specific hook
cargo run task-queue remove-hook \
    --hook-type completed \
    --receiver CONTRACT_ADDRESS_HERE \
    --task-id TASK_ID
```

The receiver contract will be notified when:
- `completed`: A task is successfully completed
- `timeout`: A task expires before completion
- `created`: A new task is added to the queue

Note: Ensure your receiver contract can properly handle the hook messages.

## Clean Up Application (Optional)

There is a global namespace for these applications, so you might get a conflict above
if you paste in YOUR_NAME_HERE after someone else did the same, without replacing
it with your favorite name. To make life easier for yourself (and others) later,
if you are not using the instance, you can remove it later.

```bash
cargo run -- wasmatic remove --name YOUR_NAME_HERE
```
