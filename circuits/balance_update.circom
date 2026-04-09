pragma circom 2.0.0;
include "poseidon";

// Balance Update Circuit for Darklake Perpetuals
//
// Proves a valid balance state transition using Poseidon commitments:
//   old_commitment = Poseidon(old_balance, old_blinding)
//   new_balance    = old_balance + delta
//   new_commitment = Poseidon(new_balance, new_blinding)
//
// The prover knows (old_balance, old_blinding, new_blinding) privately.
// The verifier sees (delta, old_commitment, new_commitment) publicly.

template BalanceUpdate() {
    signal private input old_balance;
    signal private input old_blinding;
    signal private input new_blinding;
    signal input delta;

    signal output old_commitment;
    signal output new_commitment;

    signal new_balance;
    new_balance <== old_balance + delta;

    old_commitment <== Poseidon(old_balance, old_blinding);
    new_commitment <== Poseidon(new_balance, new_blinding);
}

component main {public [delta]} = BalanceUpdate();
