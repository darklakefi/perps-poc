#!/bin/bash
# Generate a proof for mark_price >= liquidation_price using the Circom-like pipeline.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DARKLAKE="${1:-$(cd "$SCRIPT_DIR/../.." && pwd)}"

ZYGA_BIN="$DARKLAKE/zyga/rust/target/release/zyga"
PROVING_KEY="$SCRIPT_DIR/liquidation_check.zyga"
WITNESS="$SCRIPT_DIR/liquidation_witness.json"
PROOF="$SCRIPT_DIR/liquidation_proof.json"

MARK_PRICE="${MARK_PRICE:-1000000}"
LIQUIDATION_PRICE="${LIQUIDATION_PRICE:-900000}"

if [ ! -f "$PROVING_KEY" ]; then
    echo "Error: Proving key not found at $PROVING_KEY"
    echo "Run ./build_liquidation_check.sh first."
    exit 1
fi

if [ ! -f "$ZYGA_BIN" ]; then
    echo "Error: Zyga binary not found at $ZYGA_BIN"
    exit 1
fi

cat > "$WITNESS" <<EOF
{
  "mark_price": $MARK_PRICE,
  "liquidation_price": $LIQUIDATION_PRICE
}
EOF

echo "=== Liquidation Check Proof Test ==="
echo "mark_price:        $MARK_PRICE"
echo "liquidation_price: $LIQUIDATION_PRICE"

time "$ZYGA_BIN" prove \
    -s "$PROVING_KEY" \
    -w "$WITNESS" \
    -o "$PROOF"

python3 - <<PY
import json
with open("$PROOF") as f:
    data = json.load(f)
print("Public inputs:")
for key, value in data.get("public_inputs", {}).items():
    print(f"  {key}: {value}")
print()
print("Proof written to:", "$PROOF")
PY
