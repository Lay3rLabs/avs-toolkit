#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

# The minimum Rust version we support
MSRV="1.80.0"

# install rustup if not present
if ! which rustup > /dev/null; then 
    echo "Installing Rustup tooling"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
fi

# ensure we have a recent-enough version of Rust
V=$(cargo --version | cut -f2 -d' ')
if [[ $V < $MSRV ]]; then
    echo "Upgrading Rust to $MSRV"
    rustup update stable
fi

# install the wasm32-wasi target
echo "Adding WASI tooling..."
rustup target add wasm32-wasip1
cargo install cargo-component wkg

# set default registry
wkg config --default-registry wa.dev
