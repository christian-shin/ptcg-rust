//! Hoothoot (SCR): Triple Stab — flip 3 coins, 10 damage for each heads.
//!
//! Twinleaf has several `Hoothoot` classes; this port is bound to SCR.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Hoothoot@SCR",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(3), per_heads: PerHeads::DamageIs(10), ..CoinSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
