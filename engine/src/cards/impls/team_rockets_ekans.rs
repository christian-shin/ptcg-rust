//! Team Rocket's Ekans (DRI 112): Drag Down — flip a coin. If heads, your opponent's Active Pokémon is now Paralyzed. Gnaw — 10.
//!
//! On heads the Special Condition is an effect of the attack (GainCondition caused by it): Mist Energy and the like prevent it.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsEkans",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
