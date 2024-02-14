#!/bin/bash

SCRIPT=$(readlink -f "$0")
SCRIPT_DIR=$(dirname "$SCRIPT")
cd $SCRIPT_DIR/..

TESTNAME=$1
TEST_THREADS=${2:-2}

if [[ $OSTYPE == "linux-gnu"* ]] || [[ $RUNNER_OS == "Linux" ]]
then
    PLATFORM=linux
elif [[ $OSTYPE == "darwin"* ]] || [[ $RUNNER_OS == "macOS" ]]
then
    PLATFORM=darwin
else
    echo "OS not supported: ${OSTYPE:-$RUNNER_OS}"
    exit 1
fi

echo "Building canister wasm"
cargo build --target wasm32-unknown-unknown --release -p event_sink_canister_impl --locked

cd rs/integration_tests
echo "PocketIC download starting"
curl -sO https://download.dfinity.systems/ic/a7862784e8da4a97a1d608fd5b3db365de41a2d7/binaries/x86_64-$PLATFORM/pocket-ic.gz || exit 1
gzip -df pocket-ic.gz
chmod +x pocket-ic
echo "PocketIC download completed"
cd ..

cargo test --package integration_tests $TESTNAME -- --test-threads $TEST_THREADS
