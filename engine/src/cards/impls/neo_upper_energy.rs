//! Neo Upper Energy (TEF 162, ACE SPEC): provides [C]; on a Stage 2 Pokémon it provides every type of Energy but only 2
//! Energy at a time.
//!
//! A `ProvidesEnergy` declaration (two units of any type on a Stage 2 Pokémon, one [C] otherwise). Every count of "Energy
//! attached" reads the units provided (id2237, id2347; JP FAQ Neo Upper Energy), so it counts twice for Raging Bolt, Rabsca
//! and the like. A Special Energy whose text is off provides [C] only (`probe`).
use crate::spec::prelude::*;
use crate::types::{ct, Stage};

pub static SPEC: CardSpec = CardSpec {
    class: "NeoUpperEnergy",
    passives: &[Passive {
        origin: RuleSource::Energy,
        // Provides [C]; on a Stage 2 Pokémon every type of Energy, but only 2 Energy at a time.
        modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec {
            entries: &[
                ProvidedEntry { when: SlotPred::StageIs(Stage::Stage2), provides: &[ct::ANY, ct::ANY] },
                ProvidedEntry { when: SlotPred::Not(&SlotPred::StageIs(Stage::Stage2)), provides: &[ct::COLORLESS] },
            ],
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
