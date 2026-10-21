//! Baltoy (CRI 46): Continuous Spin - 30x; flip a coin until you get tails. This attack does 30 damage for each heads.
//!
//! The flips come before the damage; the damage is 30 per heads (`Flips::UntilTails`, `PerHeads::DamageIs`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Baltoy",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { flips: Flips::UntilTails, per_heads: PerHeads::DamageIs(30), ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
