//! Hoothoot (SCR): Triple Stab - flip 3 coins, 10 damage for each heads.
//!
//! The 3 flips come before the damage; the damage is 10 per heads (`PerHeads::DamageIs`). This port is the SCR card.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HoothootSCR",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(Num::Lit(3)), per_heads: PerHeads::DamageIs(10), ..CoinSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
