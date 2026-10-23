//! Pachirisu (MEG 51): Electrified Incisors - 10; during your opponent's next turn, whenever they attach an Energy card from their
//! hand to the Defending Pokémon, place 8 damage counters on that Pokémon.
//!
//! A lasting trigger on the Defending Pokémon (`Lasting::CountersOnEnergyAttached`): each Energy card its player attaches to it from
//! the hand, by the turn's attachment or by an effect, puts 80 HP of counters on it. A protection from the effects of attacks keeps the
//! effect off when the attack puts it; one that came later (a Mist Energy attached from the hand included) doesn't stop the counters:
//! the effect is already on the Pokémon (id1121, id2341). The effect is on the Pokémon, not on the player: retreating or evolving
//! clears it (id2223).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Pachirisu",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CountersOnEnergyAttached(80) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
