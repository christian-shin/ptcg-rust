//! Kangaskhan (DRI 151): Gentle Slap - 50. Dizzy Punch - 90x; flip 2 coins. This attack does 90 damage for each heads.
//!
//! The 2 flips come before the damage; 90 per heads (Cottonee PRE 7).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Kangaskhan",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(Num::Lit(2)), per_heads: PerHeads::DamageIs(90), ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
