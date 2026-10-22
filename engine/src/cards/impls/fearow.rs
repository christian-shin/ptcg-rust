//! Fearow (MEG 103): Repeating Drill - 30×; flip 5 coins. This attack does 30 damage for each heads.
//!
//! `Flips::Count(5)` + `PerHeads::DamageIs(30)` (Cottonee PRE 7).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Fearow",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(5), per_heads: PerHeads::DamageIs(30), ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
