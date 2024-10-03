#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

# Enable recursive globbing
shopt -s globstar

rm -rf ./schemas
mkdir -p ./schemas

BASEDIR=$(pwd)
for C in contracts/**/*/Cargo.toml; do
  DIR=$(dirname "$C")
  echo "Building schema for $DIR"
  (
    cd "$DIR";
    cargo schema > /dev/null;
    ls ./schema/*.json;
    cd -
  )
done
