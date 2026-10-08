//! Neo Upper Energy (TEF, ACE SPEC): provides [C]; on a Stage 2 Pokémon it
//! provides every type of Energy but only 2 Energy at a time.
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
            probe: false,
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
