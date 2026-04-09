#!/usr/bin/env python3
"""
Generate a Balance Update circuit for the Darklake perpetuals system.

Proves a valid balance state transition using Poseidon commitments:
  - old_commitment = Poseidon(old_balance, old_blinding)
  - new_balance    = old_balance + delta
  - new_commitment = Poseidon(new_balance, new_blinding)

The prover knows (old_balance, old_blinding, new_blinding) privately.
The verifier sees (old_commitment, new_commitment, delta) publicly.

This enables privacy-preserving balance tracking: the server stores only
commitments, and clients prove valid transitions without revealing balances.

Uses BN254 Poseidon (t=3, Rf=8, Rp=57, alpha=5).
"""

import sys

# BN254 scalar field prime
P = 21888242871839275222246405745257275088548364400416034343698204186575808495617

def modinv(a, p):
    if a == 0:
        raise ValueError("no inverse for 0")
    g, x, _ = extended_gcd(a % p, p)
    if g != 1:
        raise ValueError(f"no inverse: gcd={g}")
    return x % p

def extended_gcd(a, b):
    if a == 0:
        return b, 0, 1
    g, x, y = extended_gcd(b % a, a)
    return g, y - (b // a) * x, x

# ---------- Grain LFSR ----------

def grain_lfsr_init(field_size_bits, t, Rf, Rp):
    state = []
    state += [1, 0]                              # field type = prime
    state += [1, 0, 1, 0]                        # alpha = 5
    n = field_size_bits
    for i in range(12): state.append((n >> i) & 1)
    for i in range(12): state.append((t >> i) & 1)
    for i in range(10): state.append((Rf >> i) & 1)
    for i in range(10): state.append((Rp >> i) & 1)
    state += [1] * 30
    assert len(state) == 80
    return state

def grain_get_bit(state):
    new_bit = state[0] ^ state[13] ^ state[23] ^ state[38] ^ state[51] ^ state[62]
    state.pop(0)
    state.append(new_bit)
    return new_bit

def grain_get_field_element(state, n, p):
    while True:
        bits = []
        for _ in range(n):
            while grain_get_bit(state) == 0:
                grain_get_bit(state)
            bits.append(grain_get_bit(state))
        val = sum(b * (2 ** i) for i, b in enumerate(bits))
        if val < p:
            return val

def generate_round_constants(t, Rf, Rp):
    n = 254
    state = grain_lfsr_init(n, t, Rf, Rp)
    for _ in range(160):
        grain_get_bit(state)
    num_constants = (Rf + Rp) * t
    return [grain_get_field_element(state, n, P) for _ in range(num_constants)]

def generate_mds_matrix(t):
    xs = list(range(t))
    ys = list(range(t, 2 * t))
    M = []
    for i in range(t):
        row = []
        for j in range(t):
            row.append(modinv((xs[i] + ys[j]) % P, P))
        M.append(row)
    return M

# ---------- Inline Poseidon instance ----------

def emit_poseidon_instance(w, prefix, input0, input1, t, Rf, Rp, C, M):
    """Emit an inline Poseidon hash computation with the given prefix.

    MDS and round constants are referenced by global names (shared).
    State/sbox/mix signals are prefixed to make them unique per instance.
    """
    nRounds = Rf + Rp
    half_Rf = Rf // 2

    # Initial state: [0, input0, input1]
    w(f'    signal {prefix}state_0_0;')
    w(f'    {prefix}state_0_0 <== 0;')
    w(f'    signal {prefix}state_0_1;')
    w(f'    {prefix}state_0_1 <== {input0};')
    w(f'    signal {prefix}state_0_2;')
    w(f'    {prefix}state_0_2 <== {input1};')
    w('')

    for r in range(nRounds):
        is_full = (r < half_Rf) or (r >= half_Rf + Rp)

        # AddRoundConstants
        for i in range(t):
            w(f'    signal {prefix}ark_{r}_{i};')
            w(f'    {prefix}ark_{r}_{i} <== {prefix}state_{r}_{i} + rc_{r}_{i};')
        w('')

        # S-box
        if is_full:
            for i in range(t):
                w(f'    component {prefix}sbox_{r}_{i} = Sigma();')
                w(f'    {prefix}sbox_{r}_{i}.in <== {prefix}ark_{r}_{i};')
            w('')
            sbox_out = lambda i, _r=r: f'{prefix}sbox_{_r}_{i}.out'
        else:
            w(f'    component {prefix}sbox_{r}_0 = Sigma();')
            w(f'    {prefix}sbox_{r}_0.in <== {prefix}ark_{r}_0;')
            w('')
            sbox_out = lambda i, _r=r: f'{prefix}sbox_{_r}_0.out' if i == 0 else f'{prefix}ark_{_r}_{i}'

        # MDS mix
        for i in range(t):
            for j in range(t):
                w(f'    signal {prefix}mix_{r}_{i}_{j};')
                w(f'    {prefix}mix_{r}_{i}_{j} <== mds_{i}_{j} * {sbox_out(j)};')
        w('')

        # Sum and next state
        next_r = r + 1
        for i in range(t):
            w(f'    signal {prefix}mixsum_{r}_{i}_01;')
            w(f'    {prefix}mixsum_{r}_{i}_01 <== {prefix}mix_{r}_{i}_0 + {prefix}mix_{r}_{i}_1;')
            w(f'    signal {prefix}state_{next_r}_{i};')
            w(f'    {prefix}state_{next_r}_{i} <== {prefix}mixsum_{r}_{i}_01 + {prefix}mix_{r}_{i}_2;')
        w('')

    # Output is element 1 of final state
    return f'{prefix}state_{nRounds}_1'


def generate_balance_update(t, Rf, Rp, output_file=None):
    nRounds = Rf + Rp

    print(f"Generating BalanceUpdate circuit for BN254 Poseidon (t={t}, Rf={Rf}, Rp={Rp})")

    C = generate_round_constants(t, Rf, Rp)
    M = generate_mds_matrix(t)

    print(f"  Round constants: {len(C)}")
    print(f"  MDS matrix: {t}x{t}")

    lines = []
    w = lines.append

    w('pragma circom 2.0.0;')
    w('')
    w('// =============================================================')
    w('// Balance Update Circuit for Darklake Perpetuals')
    w(f'// Poseidon BN254 (t={t}, Rf={Rf}, Rp={Rp}, alpha=5)')
    w('//')
    w('// Proves: old_commitment = Poseidon(old_balance, old_blinding)')
    w('//         new_balance    = old_balance + delta')
    w('//         new_commitment = Poseidon(new_balance, new_blinding)')
    w('//')
    w('// Private: old_balance, old_blinding, new_blinding')
    w('// Public:  delta, old_commitment, new_commitment')
    w('// =============================================================')
    w('')

    # S-box template (shared by both Poseidon instances)
    w('template Sigma() {')
    w('    signal input in;')
    w('    signal output out;')
    w('    signal in2;')
    w('    signal in4;')
    w('    in2 <== in * in;')
    w('    in4 <== in2 * in2;')
    w('    out <== in4 * in;')
    w('}')
    w('')

    # Main template
    w('template BalanceUpdate() {')
    w('    // Private inputs')
    w('    signal private input old_balance;')
    w('    signal private input old_blinding;')
    w('    signal private input new_blinding;')
    w('    signal input delta;  // visibility set by component main {public [delta]}')
    w('')
    w('    // Public outputs (commitments)')
    w('    signal output old_commitment;')
    w('    signal output new_commitment;')
    w('')

    # Balance constraint
    w('    // === Balance constraint: new_balance = old_balance + delta ===')
    w('    signal new_balance;')
    w('    new_balance <== old_balance + delta;')
    w('')

    # Shared MDS matrix constants
    w(f'    // Shared MDS matrix constants ({t}x{t})')
    for i in range(t):
        for j in range(t):
            w(f'    signal mds_{i}_{j};')
    for i in range(t):
        for j in range(t):
            w(f'    mds_{i}_{j} <== {M[i][j]};')
    w('')

    # Shared round constants
    w(f'    // Shared round constants ({nRounds} rounds x {t} = {nRounds * t})')
    for r in range(nRounds):
        for i in range(t):
            ci = r * t + i
            w(f'    signal rc_{r}_{i};')
            w(f'    rc_{r}_{i} <== {C[ci]};')
    w('')

    # Poseidon instance 1: hash(old_balance, old_blinding) → old_commitment
    w('    // === Poseidon Instance 1: Poseidon(old_balance, old_blinding) ===')
    h1_out = emit_poseidon_instance(w, 'h1_', 'old_balance', 'old_blinding', t, Rf, Rp, C, M)
    w(f'    old_commitment <== {h1_out};')
    w('')

    # Poseidon instance 2: hash(new_balance, new_blinding) → new_commitment
    w('    // === Poseidon Instance 2: Poseidon(new_balance, new_blinding) ===')
    h2_out = emit_poseidon_instance(w, 'h2_', 'new_balance', 'new_blinding', t, Rf, Rp, C, M)
    w(f'    new_commitment <== {h2_out};')
    w('')

    w('}')
    w('')
    w('component main {public [delta]} = BalanceUpdate();')
    w('')

    content = '\n'.join(lines)

    if output_file:
        with open(output_file, 'w') as f:
            f.write(content)
        print(f"  Written to {output_file}")
        print(f"  Lines: {len(lines)}")
    else:
        print(content)

    return content


if __name__ == '__main__':
    t = 3
    Rf = 8
    Rp = 57

    outfile = sys.argv[1] if len(sys.argv) > 1 else None
    generate_balance_update(t, Rf, Rp, outfile)
