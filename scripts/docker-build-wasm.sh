#!/bin/bash

SCRIPT=$(readlink -f "$0")
SCRIPT_DIR=$(dirname "$SCRIPT")
cd $SCRIPT_DIR/..

GIT_COMMIT_ID=$(git rev-parse HEAD)

if [ -z "${CARGO_HOME}" ]
then
  CARGO_HOME="${HOME}/.cargo"
fi

RUSTFLAGS="--remap-path-prefix $(readlink -f ${SCRIPT_DIR}/..)=/build --remap-path-prefix ${CARGO_HOME}/bin=/cargo/bin --remap-path-prefix ${CARGO_HOME}/git=/cargo/git"
for l in $(ls ${CARGO_HOME}/registry/src/)
do
  RUSTFLAGS="--remap-path-prefix ${CARGO_HOME}/registry/src/${l}=/cargo/registry/src/github ${RUSTFLAGS}"
done

echo "CommitId: $GIT_COMMIT_ID"

docker build -t event_store --build-arg git_commit_id=$GIT_COMMIT_ID --build-arg rustflags="$RUSTFLAGS" . || exit 1

container_id=$(docker create event_store)
rm -rf wasms
mkdir wasms
docker cp $container_id:/build/.dfx/ic/canisters/event_store/event_store.wasm.gz wasms
docker rm --volumes $container_id

cd wasms
for wasm in *; do
    shasum -a 256 "$wasm"
done
cd ..
