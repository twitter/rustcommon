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


## Install Valgrind and libc debugging symbols
sudo apt-get --yes install libc6-dbg

pushd /tmp
curl -L -O https://sourceware.org/pub/valgrind/valgrind-3.16.1.tar.bz2
tar xjf valgrind-3.16.1.tar.bz2
cd valgrind-3.16.1
./configure
sudo make -j2 install
popd



# Run tests with Valgrind
test "valgrind --suppressions=build/valgrind-suppressions.supp --error-exitcode=1"

# Run tests without Valgrind
test
