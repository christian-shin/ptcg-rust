//! Magnetic Metal Energy ("Magnet Metal Energy M4", CRI): provides [M]. As
//! long as this card is attached to a [M] Pokémon, that Pokémon has no
//! Retreat Cost.
//!
//! Twinleaf: the [M] entry is pushed unless an EnergyEffect probe throws.
//! On every CheckRetreatCostEffect, if the player's Active holds this card
//! and the special energy isn't blocked, a CheckPokemonTypeEffect on the
//! Active decides: [M] → `cost = []`.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "MagnetMetalEnergy",
    passives: &[
        Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry::always(&[ct::METAL])], probe: true }) },
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
