//! Nuzleaf (TEF 4): Corkscrew Punch - 20. Comet Slap - 30x; flip 3 coins, 30 damage for each heads.
//!
//! `Flips::Count(3)` with `PerHeads::DamageIs(30)` (Cottonee PRE 7).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NuzleafTEF",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(Num::Lit(3)), per_heads: PerHeads::DamageIs(30), ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
