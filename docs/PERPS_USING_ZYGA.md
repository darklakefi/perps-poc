# Darklake Perpetuals with Zyga Zero-Knowledge Proofs

## Overview

This branch replaces the TFHE (Fully Homomorphic Encryption) based balance tracking with **Zyga zero-knowledge proofs** and **Poseidon hash commitments**. Instead of the server computing on encrypted balances, the client proves valid balance transitions without revealing actual balances.

### Why replace TFHE?

| | TFHE (before) | Zyga ZK (after) |
|---|---|---|
| **Computation** | Server computes on encrypted data | Client generates proof, server verifies |
| **Trust model** | Server holds keys, sees nothing | Client holds secrets, server sees commitments |
| **Performance** | Slow FHE operations (~seconds per op) | Fast proof generation (~3s), instant verification |
| **State** | Encrypted ciphertexts (large) | Poseidon commitments (32 bytes each) |
| **Flexibility** | Limited to supported FHE operations | Any arithmetic circuit |

## Architecture

### Commitment-Based Balance Tracking

The core idea: the server stores **Poseidon hash commitments** instead of encrypted balances.

```
Client knows:              Server stores:
  balance = 5000             commitment = Poseidon(5000, 42)
  blinding = 42              (32-byte hash, cannot reverse)
```

When the balance changes (deposit, margin deduction, funding rate), the client generates a **zero-knowledge proof** that the transition is valid:

```
Client proves:                         Server sees:
  old_commitment = Poseidon(5000, 42)    old_commitment (matches stored)
  new_balance = 5000 + 1000 = 6000      delta = 1000
  new_commitment = Poseidon(6000, 777)   new_commitment (stores this)
                                         proof (verifies correctness)

Server NEVER learns: balance=5000, balance=6000, blinding=42, blinding=777
```

### State Transition Flow

```
                    CLIENT                              SERVER
                    ======                              ======

1. Client holds:                         Server holds:
   balance, blinding                     commitment = Poseidon(balance, blinding)

2. Operation occurs (deposit/withdraw/margin):
   new_balance = balance + delta
   new_blinding = random()

3. Client generates proof:
   witness = {balance, blinding,
              new_blinding, delta}
   proof = zyga.prove(witness)

4. Client sends to server:        --->   Receives:
   (proof, delta,                        (proof, delta,
    old_commitment, new_commitment)       old_commitment, new_commitment)

5.                                       Server verifies:
                                         - proof is valid
                                         - old_commitment matches stored
                                         - delta matches expected operation
                                         Server stores new_commitment
```

## The Circuit

### `circuits/balance_update.circom`

The balance update circuit uses the Poseidon library (a circom2aoa extension):

```circom
pragma circom 2.0.0;
include "poseidon";

template BalanceUpdate() {
    // Private inputs (known only to the client/prover)
    signal private input old_balance;
    signal private input old_blinding;
    signal private input new_blinding;

    // Public input (visible to verifier/server)
    signal input delta;

    // Public outputs (commitments visible to verifier/server)
    signal output old_commitment;
    signal output new_commitment;

    // Constraint: new balance = old balance + delta
    signal new_balance;
    new_balance <== old_balance + delta;

    // Commitment: hash(balance, blinding) using Poseidon
    old_commitment <== Poseidon(old_balance, old_blinding);
    new_commitment <== Poseidon(new_balance, new_blinding);
}

component main {public [delta]} = BalanceUpdate();
```

### Circuit Properties

| Property | Value |
|----------|-------|
| Hash function | Poseidon BN254 (t=3, Rf=8, Rp=57, alpha=5) |
| Private inputs | `old_balance`, `old_blinding`, `new_blinding` |
| Public inputs | `delta` |
| Public outputs | `old_commitment`, `new_commitment` |
| R1CS constraints | 3245 |
| Witness variables | 3252 |
| Proof generation | ~3s CPU |

### Poseidon Library

The `include "poseidon"` directive provides a built-in Poseidon hash function:

```circom
hash <== Poseidon(value, salt);   // 2-input: commitment with blinding factor
hash <== Poseidon(value);         // 1-input: simple hash (salt = 0)
```

This is a **circom2aoa extension** -- standard Circom requires explicit component instantiation. The library expands the call inline during transpilation, embedding the full Poseidon permutation (65 rounds, 195 round constants, 9 MDS matrix entries) as flat arithmetic constraints.

## Toolchain Pipeline

The circuit goes through four stages to become a provable system:

```
balance_update.circom    (30 lines, human-readable circuit)
        |
        | circom2aoa         (transpiler, expands Poseidon inline)
        v
balance_update.aoa       (3248 lines, flat arithmetic constraints)
        |
        | aoac -g            (AOA compiler, generates R1CS)
        v
balance_update.r1cs.json (1.5 MB, constraint matrices + witness map)
        |
        | zyga setup         (trusted setup with Powers of Tau)
        v
balance_update.zyga      (proving key, ~160 MB)
```

### Tool Repositories

| Tool | Repository | Branch |
|------|-----------|--------|
| **circom2aoa** | [tiagoaoa/AOAlang](https://github.com/tiagoaoa/AOAlang) | `BigInt` |
| **aoac** | [tiagoaoa/AOAlang](https://github.com/tiagoaoa/AOAlang) | `BigInt` |
| **zyga** | [darklakefi/zyga](https://github.com/darklakefi/zyga) | `aoac-r1cs-integration` |

### Prerequisites

- **Python 3** (for circuit generation scripts, optional)
- **GCC** (to build circom2aoa and aoac)
- **Rust/Cargo** (to build zyga)
- **curl** (to download Powers of Tau ceremony file)

### Directory Layout

```
perps-poc/
  circuits/
    balance_update.circom   # Circuit source (checked in)
    build.sh                # Full build pipeline
    test_proof.sh           # Proof generation test
    gen_balance_update.py   # Legacy inline generator (reference only)
    .gitignore              # Ignores generated artifacts
```

## Building and Testing

### Quick Start

```bash
# Clone sibling repos (if not already present)
cd ~/Darklake
git clone git@github.com:tiagoaoa/AOAlang.git -b BigInt
git clone git@github.com:darklakefi/zyga.git -b aoac-r1cs-integration
git clone git@github.com:darklakefi/perps-poc.git -b using-zyga

# Build the circuit and proving key
cd perps-poc/circuits
./build.sh

# Generate a test proof
./test_proof.sh
```

### What `build.sh` Does

1. **Builds AOAlang tools** (`circom2aoa`, `aoac`) if not already built
2. **Builds Zyga CLI** if not already built
3. **Downloads Powers of Tau** (Hermez ceremony, power 12, ~4.8 MB) if not cached
4. **Transpiles** `balance_update.circom` to `balance_update.aoa` via `circom2aoa`
5. **Compiles** `balance_update.aoa` to `balance_update.r1cs.json` via `aoac -g`
6. **Generates proving key** `balance_update.zyga` via `zyga setup`

The build assumes AOAlang and zyga are sibling directories of perps-poc. Override with:

```bash
./build.sh /path/to/darklake/root
```

### What `test_proof.sh` Does

Generates a proof for a sample deposit scenario:

```
Scenario: Deposit 1000 into a balance of 5000
  old_balance  = 5000  (private)
  old_blinding = 42    (private)
  delta        = 1000  (public)
  new_blinding = 777   (private)
```

Creates `witness.json`, runs `zyga prove`, and displays the proof output:

```
Public inputs (visible to server):
  delta: 1000
  old_commitment: 790507697675138395682879189007304065957...
  new_commitment: 700205595636263286394564186364776524459...

Constraints: 3245
The server verifies this proof against the public inputs.
It learns only: delta=1000 and the two commitments.
The actual balance (5000 -> 6000) remains hidden.
```

## Proof Structure

The proof JSON output (`proof.json`) contains:

```json
{
  "pairing_proof": {
    "proof": {
      "a_curve": "...",        // G1 point
      "g2_b1_priv": "...",     // G2 point
      "c_curve": "...",        // G1 point
      "g1_hz_priv_base": "..." // G1 point
    },
    "scalar_opening": { ... },
    "n_constraints": 3245
  },
  "public_inputs": {
    "1": "1",
    "delta": "1000",
    "old_commitment": "79050769...",
    "new_commitment": "70020559..."
  }
}
```

The four curve points constitute the zero-knowledge proof. The verifier checks a pairing equation to confirm the proof is valid without learning any private inputs.

## Applying to Perpetuals Operations

The BalanceUpdate circuit supports any operation that changes a trader's balance by a known delta:

### Deposit

```json
{
  "old_balance": 0,
  "old_blinding": 0,
  "new_blinding": 12345,
  "delta": 10000
}
```

Server sees: `delta=10000` (deposit amount), two commitments. Stores `new_commitment`.

### Open Position (Margin Deduction)

```json
{
  "old_balance": 10000,
  "old_blinding": 12345,
  "new_blinding": 67890,
  "delta": -100
}
```

Server sees: `delta=-100` (margin deducted), verifies old commitment matches, stores new commitment. The server knows margin was deducted but not the remaining balance.

### Funding Rate Adjustment

```json
{
  "old_balance": 9900,
  "old_blinding": 67890,
  "new_blinding": 11111,
  "delta": -5
}
```

Server sees: `delta=-5` (funding payment), updates commitment. Funding rate is public, balance remains private.

### Close Position (PnL Settlement)

```json
{
  "old_balance": 9895,
  "old_blinding": 11111,
  "new_blinding": 22222,
  "delta": 150
}
```

Server sees: `delta=150` (PnL credited), updates commitment.

## Technical Details

### BN254 Field Arithmetic

All circuit arithmetic operates in the BN254 scalar field:

```
p = 21888242871839275222246405745257275088548364400416034343698204186575808495617
```

This is a 254-bit prime field. Poseidon round constants and MDS matrix entries are elements of this field (78-digit decimal numbers). The toolchain preserves full precision through the entire pipeline:

- **circom2aoa**: Stores constants as strings (no truncation)
- **aoac**: R1CS coefficients as `char[80]` strings
- **zyga**: `BigConstant(String)` in expression DAG, Fr-based witness computation

### Poseidon Parameters

| Parameter | Value |
|-----------|-------|
| Field | BN254 scalar field |
| State width (t) | 3 |
| Full rounds (Rf) | 8 |
| Partial rounds (Rp) | 57 |
| Total rounds | 65 |
| S-box | x^5 |
| MDS matrix | 3x3 Cauchy matrix |
| Round constants | 195 (generated via Grain LFSR) |
| Security level | ~128 bits |

### Powers of Tau

The trusted setup uses the Hermez Powers of Tau ceremony (power 12 = 4096 max constraints). This is a public ceremony file -- no trust assumption beyond the ceremony itself.

Downloaded from: `https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_12.ptau`

### Witness Computation

When the client generates a proof, the Zyga CLI computes 3252 intermediate witness values from just 4 inputs using **BN254 field arithmetic** (mod p). This includes:

- 9 MDS matrix constant entries (78-digit numbers)
- 195 Poseidon round constants (78-digit numbers)
- S-box outputs (x^5 in the field)
- MDS mixing products
- State values across 65 rounds, for both hash instances
- The balance constraint check

All computation happens in the Fr (scalar field) domain, not floating-point.

## Security Considerations

### What the server learns

- The **delta** (balance change amount) for each operation
- The **commitments** (Poseidon hashes) before and after
- That the proof is **valid** (the transition is consistent)

### What the server does NOT learn

- The actual **balance** at any point
- The **blinding factors** (randomness in the commitments)
- Any information beyond what's implied by the delta

### Commitment binding

Poseidon is a collision-resistant hash. The client cannot:
- Find two different (balance, blinding) pairs that produce the same commitment
- Forge a proof for an invalid transition (e.g., creating money from nothing)

### Blinding factor management

The client must:
- Generate a **fresh random blinding factor** for each new commitment
- Store the current (balance, blinding) pair locally
- Never reuse a blinding factor (prevents linking transactions)

### Negative balance prevention

The current circuit does **not** enforce `new_balance >= 0`. A malicious client could prove a transition to a negative balance (which wraps around in the field). Production deployment requires an additional **range proof** to prevent this. This can be added as an extension to the BalanceUpdate circuit.

## Future Work

### Range Proofs
Add a constraint that `new_balance` is within a valid range (e.g., 0 to 2^64). This prevents negative balance attacks. Can be implemented as a binary decomposition check in the circuit.

### Multi-Operation Circuits
Extend the circuit to handle multiple operations in a single proof (batch updates), reducing per-operation proof overhead.

### Liquidation Price Tracking
Add a separate commitment for the liquidation price, with circuits for health checks (`mark_price >= liquidation_price`) that work on committed values.

### On-Chain Verification
Deploy the Zyga verifier on Solana to verify proofs on-chain. The `public_coefficients.rs` and `proving_key.rs` generated by `zyga setup` provide the on-chain verification components.

### Position Commitments
Extend the commitment scheme to cover full position state (entry price, notional, leverage) as a Merkle tree of Poseidon hashes, enabling privacy-preserving multi-position tracking.
