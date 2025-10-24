#!/usr/bin/env bash

set -e

TOOL=git-branch-assistant
RUST_TOOL=git-branch-assistant-rust
BIN=${HOME}/local/bin
BUILT_BINARY=`pwd`/build/install/${TOOL}/bin/${TOOL}

./gradlew install
ln -fs ${BUILT_BINARY} ${BIN}/${TOOL}

cargo test
cargo install --path rust-version

