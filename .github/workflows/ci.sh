#!/bin/bash

set -e

export RUST_BACKTRACE=1

## Functions
function test {
    runner="$1"

	cargo check
	cargo build --release
	cargo test --release
}

# Run tests without Valgrind
test
