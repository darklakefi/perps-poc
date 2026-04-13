pragma circom 2.0.0;
include "poseidon";
include "comparators";

template BalanceWithdraw() {
    signal private input old_balance;
    signal private input old_blinding;
    signal private input new_blinding;
    signal input delta;

    signal output old_commitment;
    signal output new_commitment;

    component ge = GreaterEqThan(64);
    ge.a <== old_balance;
    ge.b <== delta;

    signal new_balance;
    new_balance <== old_balance - delta;

    old_commitment <== Poseidon(old_balance, old_blinding);
    new_commitment <== Poseidon(new_balance, new_blinding);
}

component main {public [delta]} = BalanceWithdraw();
