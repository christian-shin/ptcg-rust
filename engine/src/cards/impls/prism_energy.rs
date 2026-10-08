//! Prism Energy (NXD): provides [C]; on a Basic Pokémon it provides every
//! type of Energy, 1 at a time.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PrismEnergy",
    passives: &[Passive {
        origin: RuleSource::Energy,
        modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry { when: SlotPred::Basic, provides: &[crate::types::ct::ANY] }], probe: false }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
