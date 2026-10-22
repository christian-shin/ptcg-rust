//! Voltorb ex (ASC 58): Hundred-Hitting Ball - 100+; flip a coin until you get tails. This attack does 100 more damage for
//! each heads.
//!
//! The flips come before the damage; 100 more per heads (`PerHeads::DamageAdd`, Mega Kangaskhan ex MEG 104).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "VoltorbEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { flips: Flips::UntilTails, per_heads: PerHeads::DamageAdd(100), ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
