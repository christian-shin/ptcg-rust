//! Cynthia's Power Weight (DRI, tool): the Cynthia's Pokémon this card is attached to gets +70 HP.
//!
//! An `HpMod` over the Holder when its Pokémon is a Cynthia's Pokémon (the Pokémon card's tag); a Tool-effects lock turns it
//! off (JP FAQ Cynthia's Power Weight: the +70 HP is only for a Cynthia's Pokémon).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "CynthiasPowerWeight",
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::HpMod(HpModSpec { amount: Num::Lit(70), subject: SlotPred::All(&[SlotPred::Holder, SlotPred::Tag(tag::CYNTHIAS)]), guard: Cond::True }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
