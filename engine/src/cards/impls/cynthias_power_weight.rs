//! Cynthia's Power Weight (DRI, tool): the Cynthia's Pokémon this card is
//! attached to gets +70 HP.
//!
//! Twinleaf: the tool block probe is a bare ToolEffect (no
//! stadium-and-tool-no-effect check).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "CynthiasPowerWeight",
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::HpMod(HpModSpec { amount: 70, subject: SlotPred::All(&[SlotPred::Holder, SlotPred::Tag(tag::CYNTHIAS)]), guard: Cond::True }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
