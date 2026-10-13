//! Elgyem (BLK 40 / SV11B): Slight Shift — move an Energy from 1 of your opponent's Pokémon to another of their Pokémon.
//! Beam — 40.
//!
//! `Op::MoveEnergy` in `Effect` mode over the opponent's Pokémon, chosen by the attacker, one Energy, after the damage: each
//! move is an effect of the attack, so a Pokémon with Mist Energy can't be chosen as the one that receives it (JP FAQ Elgyem
//! BLK 40 / Mist Energy).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Elgyem",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::MoveEnergy(MoveEnergySpec { chooser: Who::Me, owner: Who::Opp, mode: MoveEnergyMode::Effect }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
