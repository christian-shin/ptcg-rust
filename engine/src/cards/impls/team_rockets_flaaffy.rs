//! Team Rocket's Flaaffy (DRI 73): Thunder Shock - 50; flip a coin. If heads, your opponent's Active Pokémon is now
//! Paralyzed.
//!
//! A coin after the damage; heads makes the Defending Pokémon Paralyzed (Magneton MEG 46).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsFlaaffy",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
