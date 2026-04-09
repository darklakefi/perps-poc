#!/bin/bash
# Test the Balance Update proof generation.
#
# Scenario: A trader with balance=5000 and blinding=42 deposits delta=1000.
# The proof shows that:
#   old_commitment = Poseidon(5000, 42)
#   new_commitment = Poseidon(6000, 777)   (new blinding = 777)
#   new_balance    = 5000 + 1000 = 6000
#
# The server only sees (old_commitment, new_commitment, delta=1000).
# It never learns the actual balance.
#
# Usage: ./test_proof.sh [DARKLAKE_ROOT]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DARKLAKE="${1:-$(cd "$SCRIPT_DIR/../.." && pwd)}"

ZYGA_BIN="$DARKLAKE/zyga/rust/target/release/zyga"
PROVING_KEY="$SCRIPT_DIR/balance_update.zyga"

if [ ! -f "$PROVING_KEY" ]; then
    echo "Error: Proving key not found at $PROVING_KEY"
    echo "Run ./build.sh first."
    exit 1
fi

if [ ! -f "$ZYGA_BIN" ]; then
    echo "Error: Zyga binary not found at $ZYGA_BIN"
    echo "Run: cd $DARKLAKE/zyga/rust && cargo build --release --bin zyga"
    exit 1
fi

echo "=== Balance Update Proof Test ==="
echo ""
echo "Scenario: Deposit 1000 into a balance of 5000"
echo "  old_balance  = 5000  (private)"
echo "  old_blinding = 42    (private)"
echo "  delta        = 1000  (public - deposit amount)"
echo "  new_blinding = 777   (private - fresh randomness)"
echo "  new_balance  = 6000  (computed: 5000 + 1000)"
echo ""

# Create witness JSON
cat > "$SCRIPT_DIR/witness.json" << 'EOF'
{
    "old_balance": 5000,
    "old_blinding": 42,
    "new_blinding": 777,
    "delta": 1000
}
EOF

echo "=== Generating Proof ==="
time "$ZYGA_BIN" prove \
    -s "$PROVING_KEY" \
    -w "$SCRIPT_DIR/witness.json" \
    -o "$SCRIPT_DIR/proof.json"

echo ""
echo "=== Proof Output ==="
python3 -c "
import json
with open('$SCRIPT_DIR/proof.json') as f:
    data = json.load(f)

pi = data.get('public_inputs', {})
print('Public inputs (visible to server):')
for k, v in sorted(pi.items()):
    sv = str(v)
    if len(sv) > 60:
        print(f'  {k}: {sv[:40]}...{sv[-20:]}')
    else:
        print(f'  {k}: {sv}')

proof = data.get('pairing_proof', {}).get('proof', {})
print()
print('Proof curve points:')
for k in ['a_curve', 'g2_b1_priv', 'c_curve', 'g1_hz_priv_base']:
    p = proof.get(k, '')
    if isinstance(p, str):
        print(f'  {k}: {p[:50]}...')
    elif isinstance(p, dict):
        print(f'  {k}: (affine point)')

n = data.get('pairing_proof', {}).get('n_constraints', 0)
print(f'')
print(f'Constraints: {n}')
print()
print('The server verifies this proof against the public inputs.')
print('It learns only: delta=1000 and the two commitments.')
print('The actual balance (5000 -> 6000) remains hidden.')
"

echo ""
echo "=== Test Complete ==="
