//! Pachirisu (MEG 51): Electrified Incisors - 10; during your opponent's next turn, whenever they attach an Energy card from their
//! hand to the Defending Pokémon, place 8 damage counters on that Pokémon.
//!
//! A lasting trigger on the Defending Pokémon (`Lasting::CountersOnEnergyAttached`): each Energy card its player attaches to it from
//! the hand, by the turn's attachment or by an effect, puts 80 HP of counters on it (a PlaceCounters event caused by this attack, so a
//! protection stops it). The effect is on the Pokémon, not on the player: retreating or evolving clears it (id2223).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Pachirisu",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CountersOnEnergyAttached(80) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
