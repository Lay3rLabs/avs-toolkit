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
cargo install cargo-component

# setup wkg
if [[ $OSTYPE == 'darwin'* ]]; then
  WKG_DIR="$HOME/Library/Application Support/wasm-pkg"
elif [[ $OSTYPE == 'linux'* ]]; then
  WKG_DIR="$HOME/.config/wasm-pkg"
else
  echo "Unsupported OS: $OSTYPE"
  exit 1
fi
WKG_FILE="$WKG_DIR/config.toml"

if [ ! -f "$WKG_FILE" ]; then
  echo "Creating wkg config file at $WKG_FILE"
  mkdir -p "$WKG_DIR"
  echo 'default_registry = "wa.dev"' > "$WKG_FILE"
fi
