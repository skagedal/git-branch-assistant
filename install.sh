#!/usr/bin/env bash

set -e

cargo test
cargo install --path .

