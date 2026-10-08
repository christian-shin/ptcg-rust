//! Kakuna (CRI / M4): Exoskeleton — this Pokémon takes 20 less damage from
//! attacks. Hang Down — 20.
//!
//! Twinleaf: applied on PutDamageEffect (after Weakness/Resistance, as the
//! text says; phase 4b R6: it used to be DealDamageEffect, before them, and
//! missed damage to a Benched Kakuna) during the ATTACK phase whenever this
//! card is in the target's list; the lock probe runs before the top-Pokémon
//! check.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Kakuna@CRI",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 20, subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), from_any_attack: true, ..DamageTakenSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
