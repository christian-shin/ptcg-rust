//! N's Castle (JTG, stadium): each N's Pokémon in play has no Retreat Cost.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsCastle",
    passives: &[
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec { what: BlockWhat::UseStadium }) },
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::RetreatCost(RetreatCostSpec { change: CostChange::Free, subject: SlotPred::Tag(crate::types::tag::NS), ..RetreatCostSpec::DEFAULT }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
