# Deploy Tool

`cargo run -- --help` to see all the commands

## Prerequisites

### Setup .env

1. add a `.env` file
2. give it a `LOCAL_MNEMONIC` for local deployments, `TEST_MNEMONIC` for testnet
3. see https://github.com/Lay3rLabs/layer-sdk/blob/main/GETTING-STARTED.md#wallet to create a new mnemonic and give it funds

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
cargo run deploy-contracts
```

Testnet
```bash
cargo run -- --target-env=testnet deploy-contracts
```

You'll see an output at the end with the different contract's addresses