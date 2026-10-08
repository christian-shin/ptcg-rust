//! Charmander (M2 / PFL): Nimble — if this Pokémon has no Energy attached to
//! it, it has no Retreat Cost. Live Coal — 20.
//!
//! Twinleaf: on any CheckRetreatCostEffect while this card is in the player's
//! Active slot (and is its top Pokémon) and the ability isn't blocked, a
//! CheckProvidedEnergyEffect on the Active with an empty map clears the cost.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Charmander@Charmander M2",
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
