pragma circom 2.0.0;
include "comparators";

template LiquidationCheck() {
    signal input mark_price;
    signal private input liquidation_price;

    component ge = GreaterEqThan(64);
    ge.a <== mark_price;
    ge.b <== liquidation_price;
}

component main {public [mark_price]} = LiquidationCheck();
