//! Nymble (PFL 9): Flail Around - 10x; flip 3 coins. This attack does 10 damage for each heads.
//!
//! The flips come before the damage; 10 per heads (Cottonee PRE 7).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NymblePFL",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(3), per_heads: PerHeads::DamageIs(10), ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
