//! Magnetic Metal Energy ("Magnet Metal Energy M4", CRI): provides [M]. As long as this card is attached to a [M] Pokémon, that
//! Pokémon has no Retreat Cost.
//!
//! A Special Energy: a `ProvidesEnergy` of [M] and a Retreat Cost passive. The Retreat Cost is a set "no Retreat Cost" on the Pokémon holding it when that Pokémon is [M] as the game checks its
//! type now (`SlotPred::TypeIs`, APR D-07), read by the derived Retreat Cost.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "MagnetMetalEnergy",
    passives: &[
        Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry::always(&[ct::METAL])] }) },
        // On a [M] Pokémon: no Retreat Cost.
        Passive {
            origin: RuleSource::Energy,
            modifier: Modifier::RetreatCost(RetreatCostSpec {
                change: CostChange::Free,
                subject: SlotPred::All(&[SlotPred::Holder, SlotPred::TypeIs(ct::METAL)]),
                ..RetreatCostSpec::DEFAULT
            }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
