//! Granite Cave (DRI 166, Stadium): Steven's Pokémon (both yours and your opponent's) take 30 less damage from attacks from the
//! opponent's Pokémon (after applying Weakness and Resistance).
//!
//! Full Metal Lab's form over another subject: one `Modifier::DamageTaken` (30) over the Pokémon whose top card carries the
//! "Steven's" name tag, applied only to damage from the opponent's Pokémon's attacks (a player's own recoil or Bench damage on
//! their own Steven's Pokémon isn't reduced), after Weakness and Resistance. The Stadium can't be used.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GraniteCave",
    passives: &[Passive {
        origin: RuleSource::Stadium,
        modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 30, subject: SlotPred::Tag(crate::types::tag::STEVENS), ..DamageTakenSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
