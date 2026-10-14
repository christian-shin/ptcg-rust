//! Prism Energy (NXD): provides [C]; on a Basic Pokémon it provides every
//! type of Energy, 1 at a time.
//!
//! "Basic Pokémon" is read in play (`SlotPred::Basic`): a Fossil played as a Basic Pokémon counts, as in the Kaggle
//! competition engine (cabt). The official JP FAQ reads it as the card type (a Fossil doesn't count); the user decided
//! 2026-10-13 not to follow it, for competition parity: rulings/RULES.md, "Prism Energy's 'Basic Pokémon' is the card
//! type", the only deliberate departure from the official FAQ so far.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PrismEnergy",
    passives: &[Passive {
        origin: RuleSource::Energy,
        modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry { when: SlotPred::Basic, provides: &[crate::types::ct::ANY] }] }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
