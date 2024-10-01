# Deploy Tool

`cargo run -- --help` to see all the commands

## Prerequisites

### Setup .env

1. add a `.env` file
2. give it a `LOCAL_MNEMONIC` for local deployments, `TEST_MNEMONIC` for testnet
3. don't have a mnemonic? run `cargo run wallet create` 
4. don't have funds? run `cargo run faucet tap` 

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