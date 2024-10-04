# Demo with Testnet

This shows how to deploy an AVS on the permissionless and setup stuff

## Check Testnet Connectivity

```bash
# Ensure the chain is up
curl https://rpc.layer-p.net/status

# see blocks are being made.. run a few times
curl https://rpc.layer-p.net/status | jq .result.sync_info

# ensure wasmatic opertators are set up with different addresses
curl https://op1.layer-p.net/info | jq -r .operators[0]
curl https://op2.layer-p.net/info | jq -r .operators[0]
curl https://op3.layer-p.net/info | jq -r .operators[0]
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
cargo run deploy contracts --operators wasmatic

# Copy the line that says "export TEST_TASK_QUEUE_ADDRESS" and paste it in your shell

cargo run task-queue view-queue
```

### Deploy WASI component

```bash
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

## Clean Up Application (Optional)

There is a global namespace for these applications, so you might get a conflict above
if you paste in YOUR_NAME_HERE after someone else did the same, without replacing
it with your favorite name. To make life easier for yourself (and others) later,
if you are not using the instance, you can remove it later.

```bash
cargo run -- wasmatic remove --name YOUR_NAME_HERE
```
