//! Cottonee (PRE 7): Triple Spin - 10x; flip 3 coins. This attack does 10 damage for each heads.
//!
//! The 3 flips come before the damage; the damage is 10 per heads (`PerHeads::DamageIs`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CottoneePRE",
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
