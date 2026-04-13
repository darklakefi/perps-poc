#!/bin/bash
# Build the liquidation check circuit through the full Zyga pipeline:
#   Circom -> circom2aoa -> .aoa -> aoac -g -> R1CS JSON -> zyga setup

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DARKLAKE="${1:-$(cd "$SCRIPT_DIR/../.." && pwd)}"

AOALANG="$DARKLAKE/AOAlang"
ZYGA="$DARKLAKE/zyga"
PTAU="$SCRIPT_DIR/powersOfTau28_hez_final_12.ptau"
ZYGA_BIN="$ZYGA/rust/target/release/zyga"
SOURCE="$SCRIPT_DIR/liquidation_check.circom"
AOA="$SCRIPT_DIR/liquidation_check.aoa"
R1CS="$SCRIPT_DIR/liquidation_check.r1cs.json"
OUTPUT_BASE="$SCRIPT_DIR/liquidation_check"

echo "=== Darklake Liquidation Check Build ==="
echo "  AOAlang: $AOALANG"
echo "  Zyga:    $ZYGA"
echo ""

if [ ! -f "$AOALANG/bin/circom2aoa" ] || [ ! -f "$AOALANG/bin/aoac" ]; then
    echo "Building AOAlang tools..."
    (cd "$AOALANG" && make -j)
fi

if [ ! -f "$ZYGA_BIN" ]; then
    echo "Building Zyga..."
    (cd "$ZYGA/rust" && cargo build --release --bin zyga)
fi

if [ ! -f "$PTAU" ]; then
    echo "Downloading Powers of Tau (power 12)..."
    curl -L -o "$PTAU" \
        "https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_12.ptau"
fi

echo ""
echo "=== Step 1: Circom -> AOA ==="
"$AOALANG/bin/circom2aoa" "$SOURCE" -o "$AOA"
echo "  Wrote $AOA"

echo ""
echo "=== Step 2: AOA -> R1CS JSON ==="
"$AOALANG/bin/aoac" -g "$AOA"
echo "  Wrote $R1CS"

echo ""
echo "=== Step 3: Zyga Setup ==="
"$ZYGA_BIN" setup \
    -r "$R1CS" \
    -o "$OUTPUT_BASE" \
    --ptau "$PTAU" \
    --prefix liquidation_check

echo ""
echo "=== Build Complete ==="
echo "Generated files:"
echo "  - liquidation_check.aoa"
echo "  - liquidation_check.r1cs.json"
echo "  - liquidation_check.zyga"
echo ""
echo "Run ./test_liquidation_proof.sh to generate a proof."
