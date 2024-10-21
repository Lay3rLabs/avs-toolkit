# Simple AVS Squaring example

This builds a simple WASI component that squares whatever number is input into
it, which can be compared with
[Eigenlayer's Incredible Squaring AVS Example](https://github.com/Layr-Labs/incredible-squaring-avs)
to demonstate how much less boilerplate is needed for building on Layer.

This also provides a minimal possible example to be copied, and to be used
in simple test cases. You can find a [more complete example here](https://github.com/Lay3rLabs/example-avs-oracle).

## Setup

You can automatically run this setup via `./scripts/setup_wasi.sh` ([see source](../../scripts/setup_wasi.sh))

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

This produces a Wasm component binary that can be found
in the workspace target directory (`../../target/wasm32-wasip1/release/cavs_square.wasm`).

## Unit Testing

To run the unit tests, build the component first with:

```bash
cargo component build
```

and then:

```
cargo test
```

## Deploy

Let's do a release build of the component:

```bash
cargo component build --release
```

Upload the compiled Wasm component to the Wasmatic node using the `avs-toolkit-cli` CLI tool (if you don't have it already,
`cargo install --path ../../tools/cli`).
Assign a unique name, as it is how your application is going to be distinguished. The examples below assume
the assigned name is `square`.

You'll also need to use the task address that was created when you deployed your contract.

```bash
avs-toolkit-cli wasmatic deploy --name square \
    --wasm-source ./target/wasm32-wasip1/release/cavs_square.wasm  \
    --testable \
    --task <TASK-ADDRESS>
```

## Testing Deployment

This can only be done if `--testable` flag was provided during deployment.
To test the deployed application on the Wasmatic node, you can provide `input` test data
that your application expects. The server responds with the output of the applicaton without
sending the result to the chain. For the input flag, you'll use the json input expected
by the WASI component you're testing.

```bash
avs-toolkit-cli wasmatic test --name square --input '{"x":9}'
```
