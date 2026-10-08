//! Lillie's Pearl (JTG, tool): if the Lillie's Pokémon this card is attached
//! to is Knocked Out by damage from an attack from your opponent's Pokémon,
//! that player takes 1 fewer Prize card.
//!
//! Twinleaf checks the owner's DAMAGE_DEALT_MARKER and the slot's Lillie's
//! tag on any card in the slot; the reduction applies before the ex bonus.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "LilliesPearl",
    passives: &[Passive {
        origin: RuleSource::Tool,
        // The Lillie's Pokémon Knocked Out by damage from an attack: the opponent takes 1 fewer Prize card.
        modifier: Modifier::PrizeAdjust(PrizeAdjustSpec {
            delta: -1,
            subject: SlotPred::All(&[SlotPred::Holder, SlotPred::AnyCardTag(tag::LILLIES)]),
            by_attack_damage: true,
            by_own_attack: None,
            guard: Cond::True,
            ..PrizeAdjustSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
