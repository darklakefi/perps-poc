#!/bin/bash
# Generate a proof for a balance reduction transition.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DARKLAKE="${1:-$(cd "$SCRIPT_DIR/../.." && pwd)}"

ZYGA_BIN="$DARKLAKE/zyga/rust/target/release/zyga"
PROVING_KEY="$SCRIPT_DIR/balance_withdraw.zyga"
WITNESS="$SCRIPT_DIR/withdraw_witness.json"
PROOF="$SCRIPT_DIR/withdraw_proof.json"

OLD_BALANCE="${OLD_BALANCE:-5000}"
OLD_BLINDING="${OLD_BLINDING:-42}"
DELTA="${DELTA:-1000}"
NEW_BLINDING="${NEW_BLINDING:-777}"

if [ ! -f "$PROVING_KEY" ]; then
    echo "Error: Proving key not found at $PROVING_KEY"
    echo "Run ./build_balance_withdraw.sh first."
    exit 1
fi

cat > "$WITNESS" <<EOF
{
  "old_balance": $OLD_BALANCE,
  "old_blinding": $OLD_BLINDING,
  "new_blinding": $NEW_BLINDING,
  "delta": $DELTA
}
EOF

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
PY
