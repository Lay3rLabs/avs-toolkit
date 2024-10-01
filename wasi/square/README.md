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
`wasm32-wasip2` target is not quite ready yet. So we will use `cargo-component` to target
`wasm32-wasip1` and package to use WASI Preview 2.

If haven't yet, add the WASI Preview 1 target:
```
rustup target add wasm32-wasip1
```

Install `cargo-component` and `wkg` CLIs:
```
cargo install cargo-component wkg
```

Set default registry configuration:
```
wkg config --default-registry wa.dev
```
For more information about configuration, see
the [wkg docs](https://github.com/bytecodealliance/wasm-pkg-tools).

## Usage

On your CLI, navigate to this directory, then run:
```
cargo component build --release
```

This produces a Wasm component bindary that can be found 
in the workspace target directory (`../../target/wasm32-wasip1/release/cavs_square.wasm`).

## Testing

To run the unit tests, build the component first with:
```
cargo component build
```
and then:
```
cargo test
```

## Deploy

Upload the compiled Wasm component to the Wasmatic node.
```
curl -X POST --data-binary @../../target/wasm32-wasip1/release/cavs_square.wasm http://0.0.0.0:8081/upload
```

Copy the digest SHA returned.
Choose a unique application name string and use in the placeholder below CURL commands.

```
read -d '' BODY << "EOF"
{
  "name": "{PLACEHOLDER-UNIQUE-NAME}",
  "digest": "sha256:{DIGEST}",
  "trigger": {
    "queue": {
      "taskQueueAddr": "{TASK-QUEUE-ADDR}",
      "hdIndex": 1,
      "pollInterval": 5
    }
  },
  "permissions": {},
  "envs": []
}
EOF

curl -X POST -H "Content-Type: application/json" http://0.0.0.0:8081/app -d "$BODY"
```
