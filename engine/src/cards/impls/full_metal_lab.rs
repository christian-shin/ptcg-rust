//! Full Metal Lab (TEF 148, Stadium): [M] Pokémon (both yours and your opponent's) take 30 less damage from attacks from your
//! opponent's Pokémon (after applying Weakness and Resistance).
//!
//! One `Modifier::DamageTaken` (30) over `SlotPred::TypeIs([M])`, the type after effects, applied only to damage from the
//! opponent's Pokémon's attacks, so a player's own recoil or Bench damage on their own [M] Pokémon isn't reduced. The Stadium
//! can't be used.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "FullMetalLab",
    passives: &[
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 30, subject: SlotPred::TypeIs(ct::METAL), ..DamageTakenSpec::DEFAULT }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
