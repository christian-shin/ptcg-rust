//! Joltik (WHT 33): Surprise Attack — 30; flip a coin. If tails, this attack does nothing.
//!
//! The flip comes before the damage; on tails the attack's damage is 0 (no damage calculation, APR B-02 note).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "JoltikWHT",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Coin(CoinSpec { tails: &[Step::new(damage_is(Num::Lit(0)))], ..CoinSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
