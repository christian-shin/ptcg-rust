//! Lively Stadium (SSP 180, Stadium): each Basic Pokémon in play (both yours and your opponent's) gets +30 HP.
//!
//! One `Modifier::HpMod` (+30, `SlotPred::Basic`) while this is the Stadium in play and Stadium effects aren't turned off
//! for the Pokémon. The Stadium can't be used (it has no `use_stadium`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LivelyStadium",
    passives: &[
        // Automatically active: it can't be announced and used.
        // Each Basic Pokémon in play (both yours and your opponent's) gets +30 HP.
        Passive { origin: RuleSource::Stadium, modifier: Modifier::HpMod(HpModSpec { amount: 30, subject: SlotPred::Basic, guard: Cond::True }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
