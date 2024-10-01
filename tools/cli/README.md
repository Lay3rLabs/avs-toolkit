# Deploy Tool

`cargo run -- --help` to see all the commands

## Prerequisites

### Setup .env

1. add a `.env` file
2. give it a `LOCAL_MNEMONIC` for local deployments, `TEST_MNEMONIC` for testnet
3. don't have a mnemonic? see https://github.com/Lay3rLabs/layer-sdk/blob/main/GETTING-STARTED.md#wallet to create a new one
4. funds will be sent automatically from the faucet (if `pre_fund_minimum` is larger than zero, which it is by default)

### Deploying Contracts

#### First build them:

In the project root:

```bash
scripts/optimizer.sh
```

#### Then deploy them:

In this directory:

Local

```bash
cargo run deploy contracts
```

Testnet
```bash
cargo run -- --target-env=testnet deploy contracts
```

If you want to see an output at the end with the different contract's addresses, make sure to run with --