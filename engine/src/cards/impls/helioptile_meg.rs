//! Helioptile (MEG 52): Double Scratch - 10x; flip 2 coins. This attack does 10 damage for each heads.
//!
//! The 2 flips come before the damage; the damage is 10 per heads (`PerHeads::DamageIs`, Cottonee PRE 7).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HelioptileMEG",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(2), per_heads: PerHeads::DamageIs(10), ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
