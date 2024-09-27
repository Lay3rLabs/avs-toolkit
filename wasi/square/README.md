# Simple AVS Squaring example

This builds a simple WASI component that squares whatever number is input into
it, which can be compared with
[Eigenlayer's Incredible Squaring AVS Example](https://github.com/Layr-Labs/incredible-squaring-avs)
to demonstate how much less boilerplate is needed for building on Layer.

This also provides a minimal possible example to be copied, and to be used
in simple test cases. You can find a [more complete example here](https://github.com/Lay3rLabs/example-avs-oracle).

## Setup

This requires Rust 1.80+. Please ensure you have that installed via `rustup`
before continuing.

TODO: explain what is needed for the toolchain. add the wasi target, add cargo component tools, etc.

## Usage

TODO: explain how to build the image and where the compiled WASI can be found.

Note: we can add deployment stuff, how to setup AVS and upload to Wasmatic elsewhere.
But this should give them the WASI (.wasm) file in a known location, and if they change the algorithm to y = x * x + 2, the WASI will reflect that.
