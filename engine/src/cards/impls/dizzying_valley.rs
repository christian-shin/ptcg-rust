//! Dizzying Valley (PFL 88, Stadium): Confused Pokémon (both yours and your opponent's) don't recover from that Special Condition
//! when they evolve or devolve.
//!
//! One `Modifier::KeepConditions` over every Pokémon, naming Confusion: the recoveries the rules make when a Pokémon evolves or
//! devolves (APR A-05, C-13) skip it (`derived::preserved_conditions`). Only those two recoveries: moving to the Bench still ends
//! it (APR A-03), as do the Checkup and the effects that remove it. The Stadium can't be used.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DizzyingValley",
    passives: &[Passive {
        origin: RuleSource::Stadium,
        modifier: Modifier::KeepConditions(KeepConditionsSpec { conds: &[SpecialCondition::Confused], subject: SlotPred::Any }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
