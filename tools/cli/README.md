# Deploy Tool

`cargo run -- --help` to see all the commands

You can also precompile it `cargo install --path .`.
After that, use `avs-toolkit-cli` wherever you see `cargo run --`.

## Prerequisites

### Setup .env

1. add a `.env` file
2. give it a `LOCAL_MNEMONIC` for local deployments, `TEST_MNEMONIC` for testnet
(If working with `localnode` setup grab `LOCAL_MNEMONIC` from [here](https://github.com/Lay3rLabs/layer-sdk/blob/main/localnode/wasmatic.env#L1))
3. don't have a mnemonic? run `cargo run wallet create` 
4. don't have funds? run `cargo run faucet tap` 

### Deploying Contracts

#### First build them

In the project root:

```bash
scripts/optimizer.sh
```

#### Then deploy them

In this directory:

Local

```bash
cargo run -- deploy contracts --operators wasmatic
```

Testnet

```bash
cargo run -- --target-env=testnet deploy contracts --operators wasmatic
```

If you want to see an output at the end with the different contract's addresses, make sure to run with --

Store the task queue for future use, based on the output:

```bash
# for local testing
LOCAL_TASK_QUEUE_ADDRESS=<address>

# for remote testnet
TEST_TASK_QUEUE_ADDRESS=<address>
```

### Then use them

```bash
cargo run -- task-queue view-queue

cargo run -- task-queue add-task --body '{"x": 9}' --description 'Square nine'

cargo run -- task-queue view-queue
```

If you'd rather not use environment variables to access the task address, you can also pass it as a flag.

```bash
cargo run -- task-queue add-task --address <address> --body '{"x": 9}' --description 'Square nine'
```
