//! Galarian Zigzagoon (FST 159, support card): Lick — 10; flip a coin, if
//! heads your opponent's Active Pokémon is now Paralyzed.
//!
//! Rule: after the damage a CoinFlip; on heads GainCondition(Paralyzed) on the
//! opponent's Active with the attack's cause (an attack effect, so Mist Energy
//! and the like prevent it).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "GalarianZigzagoon@Galarian Zigzagoon FST 159",
    attacks: &[AttackSpec {
        index: 0,
        // The Paralysis is an attack effect: a coin flip, then GainCondition(Paralyzed) with the attack's cause.
        steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
