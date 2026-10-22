//! Golurk (BLK 43): Double Smash - 80×; flip 2 coins. This attack does 80 damage for each heads. Golurk Hammer - 200.
//!
//! `Flips::Count(2)` + `PerHeads::DamageIs(80)` (Cottonee PRE 7).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Golurk",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(2), per_heads: PerHeads::DamageIs(80), ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
