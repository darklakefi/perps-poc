pragma circom 2.0.0;
include "comparators";

template LiquidationCheck() {
    signal input mark_price;
    signal private input liquidation_price;

    component ge = GreaterEqThan64();
    ge.a <== mark_price;
    ge.b <== liquidation_price;
    ge.out === 1;
}

component main {public [mark_price]} = LiquidationCheck();
