//! Mankey (SSP): Dual Chop — flip 2 coins, 10 damage for each heads
//! (MULTIPLE_COIN_FLIPS_PROMPT; the callback sets `effect.damage`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MankeySSPPool",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(2), per_heads: PerHeads::DamageIs(10), ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
