#!/bin/bash

../layer-sdk/localnode/stop.sh
../layer-sdk/localnode/reset_volumes.sh
../layer-sdk/localnode/run_all.sh

./scripts/optimizer.sh

avs-toolkit-cli --target=local deploy --mode verifier-simple contracts --operators wasmatic --artifacts-path artifacts/

avs-toolkit-cli --target=local wasmatic deploy --name square --wasm-source components/cavs_square.wasm --testable --task layer1kmfkh2rrtue9rpp28carhqrcsmd9ywr32s4kv5pvd73junpx5meqdxg034
