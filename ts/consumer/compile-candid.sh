SCRIPT=$(readlink -f "$0")
SCRIPT_DIR=$(dirname "$SCRIPT")
cd $SCRIPT_DIR

didc bind ./../../rs/canister/api/can.did -t ts > ./src/candid/types.d.ts
didc bind ./../../rs/canister/api/can.did -t js > ./src/candid/idl.js
