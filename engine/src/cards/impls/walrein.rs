//! Walrein (SSP): Frigid Fangs — 60; during your opponent's next turn,
//! Pokémon that have 2 or less Energy attached can't attack (a player-level
//! OpponentPokemonCannotAttackDuringTheirNextTurnEffect). Megaton Fall — 170;
//! this Pokémon also does 50 damage to itself.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Walrein",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::OppSmallEnergyCannotAttack(2) }))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(self_damage(50))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
