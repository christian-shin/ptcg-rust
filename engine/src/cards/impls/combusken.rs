//! Combusken (DRI): Combustion — 20. Double Kick — 40×; flip 2 coins.
//!
//! Double Kick flips 2 coins and sets the damage to 40 for each heads.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Combusken",
    // Double Kick: flip 2 coins; 40 damage for each heads.
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::before_damage(Op::Coin(CoinSpec {
            flips: Flips::Count(2),
            per_heads: PerHeads::DamageIs(40),
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
