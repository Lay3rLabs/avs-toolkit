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

Install `cargo-component`:
```
cargo install cargo-component
```

If you have the [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)
utility installed, `cargo component` can also be installed via a prebuilt
release artifact, saving time on the installation:

```
cargo binstall cargo-component
```

The configuration for registry mappings is in the process of getting better,
but for now, it is manual.

The default location is `$XDG_CONFIG_HOME/wasm-pkg/config.toml` on unix-like systems and
`{FOLDERID_RoamingAppData}\wasm-pkg\config.toml` on Windows. Examples of this are found below:

| Platform | Path                                            |
| -------- | ----------------------------------------------- |
| Linux    | `/home/<username>/.config`                      |
| macOS    | `/Users/<username>/Library/Application Support` |
| Windows  | `C:\Users\<username>\AppData\Roaming`           |

The configuration file is TOML and currently must be edited manually. A future release will include
an interactive CLI for editing the configuration. For more information about configuration, see
the [wkg docs](https://github.com/bytecodealliance/wasm-pkg-tools).

The recommended configuration that will work out of the box:

```toml
default_registry = "wa.dev"
```

## Usage

On your CLI, navigate to this directory, then run:
```
cargo component build --release
```

This produces a Wasm component bindary that can be found 
in the workspace target directory (`../../target/wasm32-wasip1/release/cavs_square.wasm`).


TODO:

Note: we can add deployment stuff, how to setup AVS and upload to Wasmatic elsewhere.
But this should give them the WASI (.wasm) file in a known location, and if they change the algorithm to y = x * x + 2, the WASI will reflect that.
