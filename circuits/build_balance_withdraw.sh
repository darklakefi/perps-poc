#!/bin/bash
# Build the balance-withdraw circuit through the full Zyga pipeline.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DARKLAKE="${1:-$(cd "$SCRIPT_DIR/../.." && pwd)}"

AOALANG="$DARKLAKE/AOAlang"
ZYGA="$DARKLAKE/zyga"
PTAU="$SCRIPT_DIR/powersOfTau28_hez_final_12.ptau"
ZYGA_BIN="$ZYGA/rust/target/release/zyga"
SOURCE="$SCRIPT_DIR/balance_withdraw.circom"
AOA="$SCRIPT_DIR/balance_withdraw.aoa"
R1CS="$SCRIPT_DIR/balance_withdraw.r1cs.json"
OUTPUT_BASE="$SCRIPT_DIR/balance_withdraw"

echo "=== Darklake Balance Withdraw Build ==="

if [ ! -f "$AOALANG/bin/circom2aoa" ] || [ ! -f "$AOALANG/bin/aoac" ]; then
    (cd "$AOALANG" && make -j)
fi

if [ ! -f "$ZYGA_BIN" ]; then
    (cd "$ZYGA/rust" && cargo build --release --bin zyga)
fi

if [ ! -f "$PTAU" ]; then
    curl -L -o "$PTAU" \
        "https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_12.ptau"
fi

"$AOALANG/bin/circom2aoa" "$SOURCE" -o "$AOA"
"$AOALANG/bin/aoac" -g "$AOA"
"$ZYGA_BIN" setup \
    -r "$R1CS" \
    -o "$OUTPUT_BASE" \
    --ptau "$PTAU" \
    --prefix balance_withdraw

echo "Generated:"
echo "  - balance_withdraw.aoa"
echo "  - balance_withdraw.r1cs.json"
echo "  - balance_withdraw.zyga"
