//! Jigglypuff (PFL 76): Ball Roll - 20x; flip a coin until you get tails. This attack does 20 damage for each heads.
//!
//! The flips come before the damage; 20 per heads (`Flips::UntilTails`, Baltoy CRI 46).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Jigglypuff",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { flips: Flips::UntilTails, per_heads: PerHeads::DamageIs(20), ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
