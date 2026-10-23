//! Timburr (TWM 103): Best Punch — 40; flip a coin. If tails, this attack does nothing.
//!
//! The coin is flipped before the damage; tails sets the damage to 0.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Timburr",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec { tails: &[Step::new(attack_does_nothing(Cond::True))], ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
