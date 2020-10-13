#!/bin/bash

set -e

export RUST_BACKTRACE=1
export BRANCH=$(git branch | grep \* | awk '{print $2}')

cargo check
cargo build --release
cargo test --release

echo "Getting previous benchmark results"
git checkout master
cargo bench

echo "Calculating performance change"
git checkour $BRANCH
cargo bench
