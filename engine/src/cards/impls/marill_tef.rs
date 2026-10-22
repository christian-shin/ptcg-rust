//! Marill (TEF 64): Ball Roll — 10×; flip a coin until you get tails. This attack does 10 damage for each heads. Magical Shot — 40.
//!
//! The flips come before the damage; the damage is 10 per heads (`Flips::UntilTails`, `PerHeads::DamageIs`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MarillTEF",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::UntilTails, per_heads: PerHeads::DamageIs(10), ..CoinSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
