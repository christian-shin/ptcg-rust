//! Ponyta (DRI 29): Double Headbutt - 10x; flip 2 coins. This attack does 10 damage for each heads.
//!
//! The flips come before the damage; 10 per heads (Cottonee PRE 7).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PonytaDRI",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(Num::Lit(2)), per_heads: PerHeads::DamageIs(10), ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
