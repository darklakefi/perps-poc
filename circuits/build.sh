#!/bin/bash
# Build the BalanceUpdate circuit through the full Zyga pipeline:
#   Circom → circom2aoa → .aoa → aoac -g → R1CS JSON → zyga setup
#
# Prerequisites (sibling directories under $DARKLAKE):
#   - AOAlang (BigInt branch): circom2aoa and aoac binaries
#   - zyga (aoac-r1cs-integration branch): zyga binary
#
# Usage: ./build.sh [DARKLAKE_ROOT]
#   DARKLAKE_ROOT defaults to ../../ (parent of perps-poc)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DARKLAKE="${1:-$(cd "$SCRIPT_DIR/../.." && pwd)}"

AOALANG="$DARKLAKE/AOAlang"
ZYGA="$DARKLAKE/zyga"
PTAU="$SCRIPT_DIR/powersOfTau28_hez_final_12.ptau"

echo "=== Darklake Balance Update Circuit Build ==="
echo "  AOAlang: $AOALANG"
echo "  Zyga:    $ZYGA"
echo ""

# 1. Build AOAlang tools if needed
if [ ! -f "$AOALANG/bin/circom2aoa" ] || [ ! -f "$AOALANG/bin/aoac" ]; then
    echo "Building AOAlang tools..."
    (cd "$AOALANG" && make -j)
fi

# 2. Build Zyga if needed
ZYGA_BIN="$ZYGA/rust/target/release/zyga"
if [ ! -f "$ZYGA_BIN" ]; then
    echo "Building Zyga..."
    (cd "$ZYGA/rust" && cargo build --release --bin zyga)
fi

# 3. Download Powers of Tau if needed (power 12: 2^12 = 4096 constraints)
if [ ! -f "$PTAU" ]; then
    echo "Downloading Powers of Tau (power 12)..."
    curl -L -o "$PTAU" \
        "https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_12.ptau"
fi

# 4. Generate Circom circuit
echo ""
echo "=== Step 1: Generate Circom ==="
python3 "$SCRIPT_DIR/gen_balance_update.py" "$SCRIPT_DIR/balance_update.circom"

# 5. Transpile Circom → AOA
echo ""
echo "=== Step 2: Circom → AOA ==="
"$AOALANG/bin/circom2aoa" "$SCRIPT_DIR/balance_update.circom"
# circom2aoa writes .aoa file alongside the .circom input
LINES=$(wc -l < "$SCRIPT_DIR/balance_update.aoa")
echo "  Written to balance_update.aoa ($LINES lines)"

# 6. Compile AOA → R1CS JSON
echo ""
echo "=== Step 3: AOA → R1CS JSON ==="
"$AOALANG/bin/aoac" -g "$SCRIPT_DIR/balance_update.aoa"
# aoac writes .r1cs.json alongside the .aoa file
SIZE=$(wc -c < "$SCRIPT_DIR/balance_update.r1cs.json")
echo "  Size: $SIZE bytes"

# 7. Zyga setup (generate proving key)
echo ""
echo "=== Step 4: Zyga Setup ==="
"$ZYGA_BIN" setup \
    -r "$SCRIPT_DIR/balance_update.r1cs.json" \
    -o "$SCRIPT_DIR/balance_update" \
    --ptau "$PTAU"

echo ""
echo "=== Build Complete ==="
echo "Generated files:"
echo "  - balance_update.circom     (Circom source)"
echo "  - balance_update.aoa        (AOA intermediate)"
echo "  - balance_update.r1cs.json  (R1CS constraints)"
echo "  - balance_update.zyga       (Zyga proving key)"
echo ""
echo "Run ./test_proof.sh to generate a proof."
