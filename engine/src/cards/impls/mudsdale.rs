//! Mudsdale (DRI 108): Ability Mud Coat - this Pokémon takes 30 less damage from attacks (after applying Weakness and Resistance). Heavy Impact - 130.
//!
//! A `DamageTaken` passive of 30 on this Pokémon from any attack (Bouffalant ex WHT 77).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Mudsdale",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 30, subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), from_any_attack: true, ..DamageTakenSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
