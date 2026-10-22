//! Minccino (BLK 75): Tail Slap - 20x; flip 2 coins. This attack does 20 damage for each heads.
//!
//! The flips come before the damage; 20 per heads (`PerHeads::DamageIs`, Cottonee PRE 7).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MinccinoBLK",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(Num::Lit(2)), per_heads: PerHeads::DamageIs(20), ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
