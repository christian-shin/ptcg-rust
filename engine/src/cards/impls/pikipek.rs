//! Pikipek (M5): Double Stab — flip 2 coins, 10 damage for each heads.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Pikipek",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(2), per_heads: PerHeads::DamageIs(10), ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
