#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

# Compiles all WASI components, places the output in components dir


OUTDIR="components"

rm -rf target/wasm32-wasip1/release/*.wasm "$OUTDIR"
mkdir -p "$OUTDIR"

BASEDIR=$(pwd)
for C in wasi/*/Cargo.toml; do
  DIR=$(dirname "$C")
  echo "Building WASI component in $DIR"
  (
    cd "$DIR";
    cargo component build --release
    cargo fmt
  )
done

cp target/wasm32-wasip1/release/*.wasm "$OUTDIR"

ls -l "$OUTDIR"
cd "$OUTDIR"
sha256sum -- *.wasm | tee checksums.txt