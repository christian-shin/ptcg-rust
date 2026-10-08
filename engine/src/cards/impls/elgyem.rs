//! Elgyem (BLK / SV11B): Slight Shift - move an Energy from 1 of your
//! opponent's Pokémon to another of their Pokémon. Beam - 40.
//!
//! Twinleaf: MOVE_AN_ENERGY_FROM_OPPONENTS_POKEMON_TO_ANOTHER: needs an
//! Energy card on some opponent Pokémon and at least 2 Pokémon in play, then a
//! mandatory MoveEnergyPrompt (1 transfer); each transfer is a
//! MoveOpponentEnergyEffect (an attack effect that Mist Energy & co. block).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Elgyem",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::MoveEnergy(MoveEnergySpec { chooser: Who::Me, owner: Who::Opp, mode: MoveEnergyMode::Effect }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
