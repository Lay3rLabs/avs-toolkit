#!/bin/bash

# Removes old artifacts, compile new ones (not in docker, but faster), and places in artifacts dir with checksum

# clear out old data and prepare space
rm -rf artifacts
mkdir -p artifacts
rm -f target/wasm32-unknown-unknown/release/*.wasm

# compile the wasm
RUSTFLAGS="-C link-arg=-s" cargo wasm --workspace --exclude lay3r-deploy

# place in proper outdir and add checksum
cp target/wasm32-unknown-unknown/release/*.wasm artifacts
ls -l artifacts
cd artifacts
sha256sum -- *.wasm | tee checksums.txt