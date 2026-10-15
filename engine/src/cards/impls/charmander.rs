//! Charmander (M2 / PFL): Nimble — if this Pokémon has no Energy attached to it, it has no Retreat Cost. Live Coal — 20.
//!
//! Nimble is an Ability `RetreatCost` (`Free`) over this Pokémon while it provides no Energy (Energy attached counts the Energy
//! provided, id2237).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Charmander",
    passives: &[Passive {
        origin: RuleSource::Ability,
        // No Energy attached: no Retreat Cost.
        modifier: Modifier::RetreatCost(RetreatCostSpec {
            change: CostChange::Free,
            subject: SlotPred::All(&[SlotPred::IsThisPokemon, SlotPred::NoEnergyProvided]),
            side: Side::Owner,
            ..RetreatCostSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
