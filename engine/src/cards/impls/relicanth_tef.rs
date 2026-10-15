//! Relicanth (TEF 84): Memory Dive - each of your evolved Pokémon can use any attack from its previous Evolutions. (You still
//! need the necessary Energy to use each attack.) Razor Fin - 30.
//!
//! Memory Dive is a `GrantAttacks` declaration read in the derived attack list while the Ability works: when the Active Pokémon
//! is an evolved one, the attacks of every other Pokémon card in its stack are added (scenarios
//! fix-relicanth-memory-dive-evolved-active / -basic-active).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Relicanth",
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::GrantAttacks(GrantAttacksSpec {}) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
