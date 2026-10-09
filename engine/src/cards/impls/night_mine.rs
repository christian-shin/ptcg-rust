//! Nighttime Mine (M2a): attacks used by each Tera Pokémon in play cost [C] more.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NightMine",
    passives: &[
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec::USE_STADIUM) },
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::AttackCost(AttackCostSpec { change: CostChange::Add(1), subject: SlotPred::Tag(crate::types::tag::POKEMON_TERA), ..AttackCostSpec::DEFAULT }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
