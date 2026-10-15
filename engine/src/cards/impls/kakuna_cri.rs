//! Kakuna (CRI / M4): Exoskeleton - this Pokémon takes 20 less damage from attacks. Hang Down - 20.
//!
//! `DamageTaken` 20 on this Pokémon (`Holder` and `IsThisPokemon`), from any attack, applied after Weakness and Resistance (step
//! 5 of the damage calculation, as the text says), also when Kakuna is on the Bench; it works while its Ability does.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Kakuna",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 20, subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), from_any_attack: true, ..DamageTakenSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
