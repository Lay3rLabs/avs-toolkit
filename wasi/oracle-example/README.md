# Simple AVS Oracle example

This component queries the CoinGecko API with a configured `API_KEY` env variable.
Tracks the recent BTCUSD prices and returns the average price price over the past
hour.

## Setup

This requires Rust 1.80+. Please ensure you have that installed via `rustup`
before continuing.

Even though we will be building a Wasm component that targets WASI Preview 2, the Rust
`wasm32-wasip2` target is not quite ready yet. So we will use
[`cargo-component`](https://github.com/bytecodealliance/cargo-component) to target
`wasm32-wasip1` and package to use WASI Preview 2.

If haven't yet, add the WASI Preview 1 target:
```bash
rustup target add wasm32-wasip1
```

Install `cargo-component` and `wkg` CLIs:
```bash
cargo install cargo-component wkg
```

Set default registry configuration:
```bash
wkg config --default-registry wa.dev
```
For more information about configuration, see
the [wkg docs](https://github.com/bytecodealliance/wasm-pkg-tools).

## Build

On your CLI, navigate to this directory, then run:
```bash
cargo component build --release
```

This produces a Wasm component bindary that can be found 
in the workspace target directory (`../target/wasm32-wasip1/release/oracle_example.wasm`).

Optionally, run `cargo fmt` to format the source and generated files before commiting the code.

## Unit Testing

To run the unit tests, build the component first with:
```bash
cargo component build
```
and then:
```bash
cargo test
```

## Deploying

First, let's do a release build of the component:

```bash
cargo component build --release
```

Upload the compiled Wasm component to the Wasmatic node using the `avs-toolkit-cli` CLI tool
(if you don't have it already, clone the [avs-toolkit repo](https://github.com/Lay3rLabs/avs-toolkit) and `cargo install --path ./tools/cli`).
Assign a unique name, as it is how your application is going to be distinguished. The examples below assume
the assigned name is `oracle-example`.

You'll also need to use the task address that was created when you deployed your contract.

This example integrates with the CoinGecko API to retrieve the latest BTCUSD price. You will need to sign up
and provide an API key, [see instructions](https://docs.coingecko.com/reference/setting-up-your-api-key).
Replace the `<YOUR-API-KEY>` below with your key.

```bash
avs-toolkit-cli wasmatic deploy --name oracle-example \
        --wasm-source ./target/wasm32-wasip1/release/oracle_example.wasm  \
    --testable \
    --envs "API_KEY=<YOUR-API-KEY>" \
    --task <TASK-ADDRESS>
```

## Testing Deployment

To test the deployed application on the Wasmatic node, you can use the test endpoint.
The server responds with the output of the applicaton without sending the result to the chain.

```bash
avs-toolkit-cli wasmatic test --name oracle-example --input {}
```
